mod videos;
mod config;
use actix_web::{middleware::Logger, web,App,HttpResponse,HttpServer,Responder,Result};
use std::path::Path;
use std::process::Command;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use actix_session::{Session, CookieSession};
use std::sync::RwLock;

use actix_files::NamedFile;
use tera::Tera;
use serde::{Serialize,Deserialize};
mod users;
const DB_PATH:&str = "db.json";
#[derive(Clone)]
pub struct State{
    pub config_file: config::Config,
    pub video_db: videos::VideoDB,
    pub users: users::UserVec,
    pub setup_bool:bool,
    pub use_ssl:bool,//whether or not to redirect to ssl
}
#[derive(Clone,Serialize)]
pub struct UserOut{
    pub username:String
}
impl State{
    //returns cookie if user is suscessfully authenticated
    pub fn auth_user(&mut self,username:String,password:String)->Result<String,String>{
        self.print_users();
        let auth_res = self.users.verify_user(username.clone(),password);
        if auth_res.is_ok(){
            return Ok(auth_res.unwrap());
        }
        return Err("invalid credentials".to_string())
    }
    pub fn is_auth(&self,token:String)->bool{
        return self.users.verify_token(token);
    }
    pub fn logout(&mut self,token:String)->Result<String,String>{
        return self.users.logout(token);
        //todo
    }
    pub fn add_user(&mut self,username:String,password:String,user_token:String)->Result<String,String>{
        if self.users.verify_token(user_token){
            return self._add_user(username,password);
        }
        return Err("not authorized".to_string());
    }
    fn _add_user(&mut self, username:String,password:String)->Result<String,String>{
        self.users.add_user(username,password);
        let res = self.write();
        return res;
    }
    pub fn get_videos(&self,user_token:String)->Result<Vec<videos::VideoHtml>,String>{
        if self.is_auth(user_token){ 
            return Ok(self.video_db.get_vid_html_vec("/vid_html/".to_string(),"/thumbnails/".to_string()));
        }
        else{
		    return Err("not authorized".to_string());
        }
    }
	pub fn get_vid_html(&self,user_token:String,video_name:String)->Result<videos::VideoHtml,String>{
		if self.users.verify_token(user_token){
                    let res = self.video_db.get_vid_html("/videos/".to_string(),
                        "/thumbnails/".to_string(),video_name);
                    if res.is_ok(){
                        return Ok(res.ok().unwrap());
                    }
                    else{
                        return Err(res.err().unwrap());
                    }
		}else{
			return Err("not authorized".to_string())
		}
	}
        pub fn get_vid_path(&self,user_token:String,video_name:String)->Result<String,String>{
            if self.is_auth(user_token){
                let res = self.video_db.get_vid_path(video_name);
                if res.is_ok(){
                    return Ok(res.ok().unwrap());
                }
                else{
                    return Err(res.err().unwrap());
                }
                return Err("file not found".to_string());
            }else{
                return Err("not authorized".to_string());
            }
        }
	pub fn get_vid_dir(&self)->String{
		return self.config_file.videos.video_path.clone();
	}
        pub fn get_thumb_dir(&self)->String{
            return self.config_file.videos.thumbnails.clone();
        }
        pub fn is_setup(&self)->bool{
            return self.setup_bool;
        }
        fn set_thumb_res(&mut self,thumb_res: u32)->Result<String,String>{
            self.config_file.thumb_res=thumb_res;
            let res = config::write_conf(self.config_file.clone());
            if !res.is_ok(){
                return Err("failed to write config".to_string());
            }
            let video_res = videos::new(self.config_file.videos.video_path.clone(),
                self.config_file.videos.thumbnails.clone(),DB_PATH.to_string(),thumb_res);
            if video_res.is_ok(){
                self.video_db=video_res.ok().unwrap();
            }else{
                return Err(video_res.err().unwrap());
            }
            return Ok("sucess".to_string());
        }
        pub fn set_thumb_res_auth(&mut self,token:String,thumb_res:u32)->Result<String,String>{
            if self.is_auth(token){
                let final_res = self.set_thumb_res(thumb_res);
                if final_res.is_ok(){
                    return Ok("sucess".to_string());
                }else{
                    return final_res;
                }
            }
            return Err("permission denied".to_string());
        }
        pub fn setup(&mut self,video_dir:String, 
                     username:String, 
                     password:String,
                     thumb_res: u32)->Result<String,String>{
            if self.is_setup(){
                return Err("already setup".to_string());
            }
            let reload_res = self.reload_server(video_dir,thumb_res);
            let add_user_res = self._add_user(username,password);

            if reload_res.is_ok() && add_user_res.is_ok(){
                self.setup_bool=true;
                return Ok("Sucess".to_string());
            }else{
                return Err("failed to add user".to_string());
            }

        }
        pub fn reload_server(&mut self,video_dir:String,thumb_res:u32
                     )->Result<String,String>{
            self.config_file.videos.video_path=video_dir.clone();
            self.config_file.videos.thumbnails="thumbnails".to_string();
            self.config_file.thumb_res=thumb_res;
            let video_res = videos::new(video_dir.clone(),"thumbnails".to_string(),
                DB_PATH.to_string(),thumb_res);
            if video_res.is_ok(){
                self.video_db=video_res.ok().unwrap()
            }else{
                return Err(video_res.err().unwrap());
            }
            return Ok("done".to_string());
        }
        pub fn get_users(&self,token:String)->Result<Vec<UserOut>,String>{
            if self.is_auth(token){
                let mut out:Vec<UserOut> = Vec::new();
                for user in self.users._users.clone(){
                    out.push(UserOut{username:user.name.clone()});
                }
                return Ok(out);
            }else{
                return Err("not authorized".to_string());
            }
        }
    pub fn print_users(&self){
        println!("Users: ");
        println!("{}",self.users.print_users());    
    }
	fn write(&mut self)->Result<String,String>{
		let temp_user = self.users.ret_conf_users();
		let mut users_write:Vec<config::User>=Vec::new();
		for user in temp_user{
			users_write.push(config::User{
				username: user.username,
				passwd: user.password
			});
		}
		self.config_file.users=users_write;
		let res = config::write_conf(self.config_file.clone());
                if res.is_ok(){
                    return Ok("sucess".to_string());;
                }else{
                    return Err("error in writing".to_string()); 
                }
	}
}
lazy_static!{
	pub static ref TERA: Tera = {
		let tera = compile_templates!("templates/**/*");
		tera
	};
}
//used to declare things that will be set in the cli args
struct StartupOptions{
    use_ssl:bool,//whether or not to redirect to https
}
fn init_state(startup_otions:StartupOptions)->Result<State,String>{
    let temp_cfg=config::load_config();
    if temp_cfg.is_ok(){
        let cfg = temp_cfg.ok().unwrap();
        let vid_dir=cfg.videos.video_path.clone();
        let video_res = videos::new(vid_dir,"thumbnails".to_string(),
            DB_PATH.to_string(),cfg.thumb_res);
        if video_res.is_ok(){
            let mut out=State{
                config_file: cfg.clone(),
                video_db: video_res.ok().unwrap(),
                users: users::new(),
                setup_bool: true,
                use_ssl:startup_otions.use_ssl,
            };
            for user in cfg.users.clone(){
                let res = out.users.load_user(user.username,user.passwd);
                if res.is_err(){
                    println!("failed to add user");
                }
            }
            return Ok(out);
        }else{
            return Err(video_res.err().unwrap());
        }
    }else{
        return Err(temp_cfg.err().unwrap());
    }
    return Err("unreachable".to_string());

}
//returns an empty state
fn empty_state(startup_otions:StartupOptions)->State{
    return State{
        config_file: config::empty(),
        video_db: videos::empty(),
        users: users::new(),
        setup_bool: false,
        use_ssl: startup_otions.use_ssl
    }
}
fn make_ssl_key(){
    if !Path::new("key.pem").exists() || !Path::new("cert.pem").exists(){
        println!("making ssl");
        let res = Command::new("openssl").arg("req").arg("-x509").arg("-newkey").arg("rsa:4096")
            .arg("-nodes").arg("-keyout").arg("key.pem").arg("-out").arg("cert.pem")
            .arg("-days").arg("365").arg("-subj").arg("/CN=localhost").output();
        println!("done with ssl");
    }
}
pub fn run_webserver(state_in:&mut State,use_ssl:bool){
    let video_dir = state_in.get_vid_dir();
    let thumb_dir= state_in.get_thumb_dir();
    let temp_state = RwLock::new(state_in.clone());
    let shared_state = web::Data::new(temp_state);
    // load ssl keys
    /*
    let mut builder =
        SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();
    */
    std::env::set_var("RUST_LOG", "my_errors=debug,actix_web=info");
    std::env::set_var("RUST_BACKTRACE", "1");
	env_logger::init();
    let http_server = HttpServer::new(move || {
        App::new().wrap(
            CookieSession::signed(&[0; 32]) // <- create cookie based session middleware
                    .secure(false)
            ).wrap( Logger::default())
			.register_data(shared_state.clone())
            .route("/api/login",web::post().to(login))
	    .route("/api/videos",web::get().to(get_videos))
	    .route("/api/add_user",web::post().to(add_user))
            .route("/api/get_user",web::get().to(get_users))
            .route("/vid_html/{name}",web::get().to(vid_html))
            .route("/settings",web::get().to(settings))
            .route("/", web::get().to(index))
            .route("/login",web::get().to(login_html))
            .route("/setup",web::get().to(setup))
            .route("/api/setup",web::post().to(api_setup))
            .route("/api/logout",web::post().to(logout_api))
            .route("/api/settings",web::post().to(settings_api))
            .route("/videos/{video_name}",web::get().to(video_files))
            .service(actix_files::Files::new("/static","./static/"))
            .service(actix_files::Files::new("/thumbnails",thumb_dir.clone()))
			
    });
    if use_ssl{
        // load ssl keys
        make_ssl_key();
        let mut builder =
            SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        builder
            .set_private_key_file("key.pem", SslFiletype::PEM)
            .unwrap();
        builder.set_certificate_chain_file("cert.pem").unwrap();
        http_server.bind_ssl("0.0.0.0:8443",builder).unwrap().run().unwrap();
    }
    else{
        http_server.bind("0.0.0.0:8088").unwrap()
        .run()
        .unwrap();
    }
}
//starts the web server, if use_ssl is true than all requests will be sent through https
pub fn init(use_ssl:bool){
    let mut state_res = init_state(StartupOptions{use_ssl:use_ssl});
    if state_res.is_ok(){
        run_webserver(&mut state_res.ok().unwrap(),use_ssl);
    }else{
        let mut state = empty_state(StartupOptions{use_ssl:use_ssl});
        run_webserver(&mut state,use_ssl);
    }
}
#[derive(Deserialize)]
struct UserReq{
    username: String,
    password: String,
}
fn login(info: web::Json<UserReq>, data:web::Data<RwLock<State>>,session:Session)-> Result<String>{
    println!("Processed Username: {} Password: {}",info.username,info.password);
	let mut state_data=data.write().unwrap();
    let auth=state_data.auth_user(info.username.clone(),info.password.clone());
    if auth.is_ok(){
        println!("Authenticated Username: {} Password: {}",info.username,info.password);
        let token = auth.unwrap();
        println!("token: {}",token.clone());
        let res = session.set("token",token);
        if res.is_ok(){
            return Ok("logged in sucessfully".to_string());
        }else{
            return Ok("failed to set cookie".to_string());
        }
    }
    else{
        println!("Denied Username: {} Password: {}",info.username,info.password);
        return Ok("Login Failed".to_string());

    }
}
fn add_user(info:web::Json<UserReq>,data:web::Data<RwLock<State>>,session:Session)->Result<String>{
    let token = session.get("token").unwrap().unwrap();
    let username = info.username.clone();
    let password = info.password.clone();
    let mut state_data = data.write().unwrap();
    state_data.print_users();
    let res = state_data.add_user(username.clone(),password.clone(),token);
    if res.is_ok(){
        println!("Added Username: {} Password: {}",username,password);
        return Ok("sucess".to_string());
    }
    return Ok("failed".to_string());
}
#[derive(Serialize)]
pub struct UsersApi{
    users:Vec<UserOut>
}
pub fn get_users(data: web::Data<RwLock<State>>,session:Session)->impl Responder{
    let token = session.get("token");
    if token.is_ok(){
        let state = data.read().unwrap();

        let out = state.get_users(token.unwrap().unwrap());
        if out.is_ok(){
            let body = serde_json::to_string(&UsersApi{users:out.unwrap()});
            if body.is_ok(){
                let message_body = body.unwrap();
                println!("message: {}",message_body);
                return HttpResponse::Ok().body(message_body);
            }else{
                return HttpResponse::InternalServerError().body("");
            }
        }else{
            return HttpResponse::Unauthorized().body("");
        }
    }else{
        return HttpResponse::Unauthorized().body("");
    }
}
fn get_videos(data:web::Data<RwLock<State>>,session:Session)->impl Responder{
	let token = session.get("token").unwrap().unwrap();
	let state_data = data.read().unwrap();
	let videos=state_data.get_videos(token);
	let out=serde_json::to_string(&videos).unwrap();
	return HttpResponse::Ok().body(out);	
}
#[derive(Serialize)]
struct Index{
	videos: Vec<videos::VideoHtml>
}
//todo redirect to https. I need to figure out how to do that
//Current Ideas: detect if user is on http, if on http redirect to https
pub fn index(data:web::Data<RwLock<State>>, session:Session)->impl Responder{
    let state_data = data.read().unwrap();
    if !state_data.is_setup(){
        println!("is not setup");
        return HttpResponse::TemporaryRedirect().header("location", "/setup").finish();
    }
    println!("getting token");
    let temp = session.get("token");
    let mut token:String="".to_string();
    if temp.is_ok(){
        let temp_token = temp.ok().unwrap();
        if temp_token.is_some(){
            token=temp_token.unwrap();
        }
    }
    println!("getting state data");
    let index_data = state_data.get_videos(token); 
    if index_data.is_ok(){
	    let index_data=Index{
	        videos:index_data.ok().unwrap()
	    };
	    let out_data = TERA.render("home.jinja2",&index_data);
	    if out_data.is_ok(){
		    return HttpResponse::Ok().body(out_data.unwrap());
	    }else{
		    println!("data not rendered");
	    }
    }
    else{
        return HttpResponse::TemporaryRedirect().header("location", "/login").finish();
    }

    HttpResponse::Ok().body("".to_string())
        
}
pub fn setup(data:web::Data<RwLock<State>>,session:Session)->impl Responder{
        let render_data = TERA.render("setup.jinja2",&EmptyStruct{}); 
        let state = data.read();
        if render_data.is_ok() && !state.unwrap().is_setup(){

	    return HttpResponse::Ok().body(render_data.unwrap());
        }
            return HttpResponse::TemporaryRedirect().header("Location","/setup").finish();
}
pub fn settings(data:web::Data<RwLock<State>>,session:Session)->impl Responder{
    let render_data=TERA.render("settings.jinja2",&EmptyStruct{});
    let token_res = session.get("token");
    if token_res.is_ok(){
        let state = data.read();
        if render_data.is_ok() && state.unwrap().is_auth(token_res.unwrap().unwrap()){
            return HttpResponse::Ok().body(render_data.unwrap());
        }else{
            return HttpResponse::TemporaryRedirect().header("Location","/login").finish();
        }
    }else{
        return HttpResponse::TemporaryRedirect().header("Location","/login").finish();
    }
}
#[derive(Serialize,Deserialize)]
pub struct SettingsStruct{
    action: String,
    args: String,
}
#[derive(Serialize,Deserialize)]
pub struct SettingsAddUserStruct{
    username:String,
    password:String,
}
pub fn settings_api(info: web::Json<SettingsStruct>,data:web::Data<RwLock<State>>,session:Session)->Result<String>{
    if info.action=="set_resolution".to_string(){
        let temp_res = info.args.parse::<u32>();
        if temp_res.is_ok(){
            let mut state = data.write().unwrap();
            let token_res = session.get("token");
            if token_res.is_ok(){
                let final_res = state.set_thumb_res_auth(token_res.unwrap().unwrap(),temp_res.unwrap());
                if final_res.is_ok(){
                    return Ok("sucess".to_string());
                }else{
                    return Ok("failed to set thumbnail".to_string());
                }
            }else{
                return Ok("not authorized".to_string());
            }
        }else{
            return Ok("resolution not found".to_string());
        }

    }else{
        return Ok("action not found".to_string());
    }
}
#[derive(Serialize,Deserialize)]
struct SetupStruct{
    video_dir:String,
    username:String,
    password:String,
    thumb_res:u32,
}
fn api_setup(info: web::Json<SetupStruct>, data:web::Data<RwLock<State>>,
             session:Session)->Result<String>{
    let mut state_data = data.write().unwrap();
    let res =  state_data.setup(info.video_dir.clone(),info.username.clone(),info.password.clone(),info.thumb_res);
    if res.is_ok(){
        return Ok("Sucess".to_string());
    }else{
        return Ok(res.err().unwrap());
    }
}
fn logout_api(into: web::Json<EmptyStruct>,session:Session,data:web::Data<RwLock<State>>)->Result<String>{
    let mut state_data=data.write().unwrap();
    let token_res = session.get("token");
    if token_res.is_ok(){
        let token:String = token_res.ok().unwrap().unwrap();
        let final_res = state_data.logout(token);
            if final_res.is_ok(){
                return Ok("Sucess".to_string());
            }else{
                return Ok("failed to logout".to_string());
            }
    }else{
        return Ok("failed to get token".to_string());
    }
}
#[derive(Deserialize,Serialize)]
struct EmptyStruct{

}
pub fn login_html(data:web::Data<RwLock<State>>, session:Session) -> impl Responder{
    println!("ran redirect");
    let state_data = data.read().unwrap();
    let html = TERA.render("login.jinja2",&EmptyStruct{});
    if html.is_ok(){
        return HttpResponse::Ok().body(html.unwrap());
    }
    else{
        println!("failed to render body");
        return HttpResponse::InternalServerError().body("");
    }
}
pub fn vid_html(data:web::Data<RwLock<State>>,session:Session,path: web::Path<(String,)>)->HttpResponse{

	let token:String = session.get("token").unwrap().unwrap();
	let vid_name:String = path.0.clone();
	let state_data = data.write().unwrap();
	let vid_res = state_data.get_vid_html(token,vid_name.clone());
	if vid_res.is_ok(){

		let vid:videos::VideoHtml = vid_res.unwrap();
		let data=TERA.render("video.jinja2",&vid);
		if data.is_ok(){
			return HttpResponse::Ok().body(data.unwrap());
		}else{
			println!("did not process template correctly");
		}
	}
	else{
		println!("did not get video");
	}
	//then use videos.jinja2 to create the data and return it
		
    HttpResponse::Ok().body(vid_name)
}
pub fn video_files(data:web::Data<RwLock<State>>,session:Session,
                path:web::Path<(String,)>)-> impl Responder{
    let token_res = session.get("token");
    let state_data = data.read().unwrap();
    let vid_name:String = path.0.clone();
    println!("vid_name: {}",vid_name);
    if token_res.is_ok(){
        let token = token_res.ok().unwrap().unwrap();
        let file_path = state_data.get_vid_path(token,vid_name);
        if file_path.is_ok(){
            let file_path_out:String = file_path.unwrap();

            println!("file path: {}",file_path_out);
            let file_res = NamedFile::open(file_path_out);
            if file_res.is_ok(){
                return file_res.unwrap();
            }
            else{
                println!("file error: {}",file_res.err().unwrap());
                return NamedFile::open("empty.txt").unwrap();

            }
        }else{
            println!("file error: {}",file_path.err().unwrap());
            return NamedFile::open("empty.txt").unwrap();
        }
    }else{
        println!("video error: {}",token_res.err().unwrap());
        return NamedFile::open("empty.txt").unwrap();
    }

        return NamedFile::open("empty.txt").unwrap();
}
