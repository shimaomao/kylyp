use diesel;
use diesel::prelude::*;
use rocket_contrib::Template;
use rocket::request::{self,Form, FlashMessage,FromRequest,Request};
use rocket::response::{Redirect,Flash};
use model::user::{User,NewUser};
use model::article::{Article,Comment};
use rocket::http::{Cookie, Cookies};
use rocket::http::RawStr;
use std::collections::HashMap;
use rocket::outcome::IntoOutcome;
use chrono::prelude::*;
use handler::content::{Rarticle,UserComment,UserMessage,get_user_info,get_user_articles,get_user_comments,get_user_blogs,get_user_messages,get_unread_message_count,update_user_message};
use chrono::{DateTime,Utc};
use model::db::ConnDsl;
use model::pg::ConnPg;
use diesel::pg::PgConnection;
use bcrypt::{DEFAULT_COST, hash, verify};

#[derive(Debug,Serialize)]
pub struct Uid {
    id: i32,
}
#[derive(FromForm,Debug)]
pub struct DataCount {
    count: i32,
}
#[derive(Debug)]
pub struct UserOr(pub String);
#[derive(Debug)]
pub struct UserId(pub i32);

impl<'a, 'r> FromRequest<'a, 'r> for UserOr {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<UserOr, ()> {
        request.cookies()
            .get_private("username")
            .and_then(|cookie| cookie.value().parse().ok())
            .map(|id| UserOr(id))
            .or_forward(())
    }
}
impl<'a, 'r> FromRequest<'a, 'r> for UserId {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<UserId, ()> {
        request.cookies()
            .get_private("user_id")
            .and_then(|cookie| cookie.value().parse().ok())
            .map(|id| UserId(id))
            .or_forward(())
    }
}

#[derive(FromForm)]
struct UserRegister {
    email: String,
    username: String,
    password: String,
    password2: String,
}

#[derive(FromForm,Debug)]
struct UserLogin {
    username: String,
    password: String,
}
#[derive(Serialize)]
struct UserInfo {
    this_user: Option<User>,
    user_articles: Vec<Rarticle>,
    user_comments: Vec<UserComment>,
    user_blogs: Vec<Rarticle>,
    user_messages: Vec<UserMessage>,
    username: String,
    user_id: i32,
    count: i32,
}

#[get("/<u_id>", rank = 4)]
pub fn user_page(conn_pg: ConnPg, conn_dsl: ConnDsl, u_id: i32) -> Template {
        let a_user = get_user_info(&conn_dsl, u_id);
        let articles = get_user_articles(&conn_pg, u_id);
        let comments = get_user_comments(&conn_pg, u_id);
        let blogs = get_user_blogs(&conn_pg, u_id);
        let messages = get_user_messages(&conn_pg, u_id);
        let context = UserInfo {
            this_user: a_user,
            user_articles: articles,
            user_comments: comments,
            user_blogs: blogs,
            user_messages: messages,
            username: "".to_string(),
            user_id: 0,
            count:0,
        };
        Template::render("user", &context)
}
#[get("/<u_id>", rank = 3)]
pub fn user_page_login(conn_pg: ConnPg, conn_dsl: ConnDsl, u_id: i32,user: UserOr, user_id: UserId) -> Template {
        let a_user = get_user_info(&conn_dsl, u_id);
        let articles = get_user_articles(&conn_pg, u_id);
        let comments = get_user_comments(&conn_pg, u_id);
        let blogs = get_user_blogs(&conn_pg, u_id);
        let messages = get_user_messages(&conn_pg, u_id);
        let unread_count = get_unread_message_count(&conn_pg, u_id);
        let context = UserInfo {
            this_user: a_user,
            user_articles: articles,
            user_comments: comments,
            user_blogs: blogs,
            user_messages: messages,
            username: user.0,
            user_id: user_id.0,
            count: unread_count,
        };
        Template::render("user", &context)
}
#[get("/<u_id>?<date_count>")]
pub fn user_page_login_message(conn_pg: ConnPg, conn_dsl: ConnDsl, u_id: i32,user: UserOr, user_id: UserId,date_count: DataCount) -> Template {
        let a_user = get_user_info(&conn_dsl, u_id);
        let articles = get_user_articles(&conn_pg, u_id);
        let comments = get_user_comments(&conn_pg, u_id);
        let blogs = get_user_blogs(&conn_pg, u_id);
        let messages = get_user_messages(&conn_pg, u_id);
        if date_count.count != 0 {
            let read_count = date_count.count;
            let unread_count = get_unread_message_count(&conn_pg, u_id);
            update_user_message(&conn_pg,u_id, read_count);
            let context = UserInfo {
                this_user: a_user,
                user_articles: articles,
                user_comments: comments,
                user_blogs: blogs,
                user_messages: messages,
                username: user.0,
                user_id: user_id.0,
                count: unread_count,

            };
            Template::render("user", &context)
        } else {
            let unread_count = get_unread_message_count(&conn_pg, u_id);
            let context = UserInfo {
                this_user: a_user,
                user_articles: articles,
                user_comments: comments,
                user_blogs: blogs,
                user_messages: messages,
                username: user.0,
                user_id: user_id.0,
                count: unread_count,

            };
            Template::render("user", &context)
        }
}


#[get("/register", rank = 2)]
pub fn register(flash: Option<FlashMessage>) -> Template {
    let mut context = HashMap::new();
    if let Some(ref msg) = flash {
        context.insert("flash", msg.msg().to_string());
    }
    Template::render("register", &context)
}
#[get("/register")]
pub fn login_register(user: UserOr) -> Template {
    let mut context = HashMap::new();
    context.insert("username", user.0);
    Template::render("index", &context)
}
// #[get("/register")]
// pub fn login_register(user: UserOr) -> Redirect {
//     Redirect::to(&*("/index/".to_string() + &*user.0.to_string()))
// }

#[get("/login", rank = 2)]
pub fn login(flash: Option<FlashMessage>) -> Template {
    let mut context = HashMap::new();
    if let Some(ref msg) = flash {
        context.insert("flash", msg.msg().to_string());
    }
    Template::render("login", &context)
}
// ----------------method 1--------------
#[get("/login")]
pub fn login_user(user: UserOr) -> Template {
    let mut context = HashMap::new();
    context.insert("username", user.0);
    Template::render("index", &context)
}
// ----------------method 2--------------
// #[get("/login")]
// pub fn login_user(user: UserOr) -> Redirect {
//     Redirect::to(&*("/user/".to_string() + &*user.0.to_string()))
// }
  
#[post("/register",data = "<user_form>")]
fn register_post(conn_dsl: ConnDsl, user_form: Form< UserRegister>) -> Result<Redirect, String> {
    let post_user = user_form.get();
    use utils::schema::users;
    
    if &post_user.password == &post_user.password2 {
            let hash_password = match hash(&post_user.password, DEFAULT_COST) {
                Ok(h) => h,
                Err(_) => panic!()
            };
            let new_user = NewUser {
                email: &post_user.email,
                username: &post_user.username,
                password: &hash_password,
                created_at: Utc::now(),
            };
            diesel::insert(&new_user).into(users::table).execute(&*conn_dsl).expect("User is  Exist!");
            Ok(Redirect::to("/user/login"))
    }else {
        Err("password != password2".to_string())
    }
}
// -------------- method 1-------------
// #[post("/login", data = "<user_form>")]
// fn login_post(conn_pg: ConnPg, mut cookies: Cookies, user_form: Form<UserLogin>) -> Flash<Redirect> {
//     let post_user = user_form.get();
//     let mut uid = Uid {id : 0};
//     for row in &conn_pg.query("SELECT id FROM users WHERE username =$1  AND password = $2", &[&post_user.username,&post_user.password]).unwrap() {
//         uid = Uid {
//             id : row.get(0),
//         };
//     }
//     if uid.id != 0 {
//             cookies.add_private(Cookie::new("user_id",uid.id.to_string() ));
//             cookies.add_private(Cookie::new("username",post_user.username.to_string() ));
//             Flash::success(Redirect::to("/"), "Successfully logged in")
            
//     }else {
//             Flash::error(Redirect::to("/user/login"), "username or password Incorrect")
//     } 
// }

// -------------- method 2 -------------
#[post("/login", data = "<user_form>")]
fn login_post(conn_dsl: ConnDsl, mut cookies: Cookies, user_form: Form<UserLogin>) -> Flash<Redirect> {
    use utils::schema::users::dsl::*;
    let post_user = user_form.get();
    let user_result =  users.filter(&username.eq(&post_user.username)).load::<User>(&*conn_dsl);
    let login_user = match user_result {
        Ok(user_s) => match user_s.first() {
            Some(a_user) => Some(a_user.clone()),
            None => None,
        },
        Err(_) => None,
    };
    match login_user {
        Some(login_user) => {
            match verify(&post_user.password, &login_user.password) {
                Ok(valid) => {
                    cookies.add_private(Cookie::new("username",post_user.username.to_string() ));
                    cookies.add_private(Cookie::new("user_id",login_user.id.to_string() ));
                    Flash::success(Redirect::to("/"), "Successfully logged in")
                },
                Err(_) => Flash::error(Redirect::to("/user/login"), "Incorrect Password"),
            }
        },
        None => Flash::error(Redirect::to("/user/login"), "Incorrect Username"),
    }
}

#[get("/logout")]
pub fn logout(mut cookies: Cookies) -> Flash<Redirect> {
    cookies.remove_private(Cookie::named("username"));
    cookies.remove_private(Cookie::named("user_id"));
    Flash::success(Redirect::to("/user/login"), "Successfully logged out.")
}

