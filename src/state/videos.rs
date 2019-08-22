use serde::{Deserialize,Serialize};
use std::path::Path;
use std::thread;
use std::sync::mpsc::channel;
use std::sync::{Arc,Mutex,RwLock};

mod thumbnail;
mod db;
#[derive(Clone,Serialize,Deserialize)]
pub struct VideoHtml{
    pub name: String,
    pub url: String,
    pub thumbnail_url: String,
    pub html_url:String,
    pub path:String,
}
#[derive(Clone,Serialize,Deserialize)]
pub struct VideoRatingData{
    pub star_rating:u32,//star rating (eg 5 or 4 stars)
    pub rating:String,//normal rating (eg pg, pg13)
    pub description:String,//Dexcription Of video
}
#[derive(Clone)]
pub struct VideoDB{
    database: db::FileDB,
    thumb_dir:String,
    thumb_res:u32,
}
#[derive(Clone,Serialize,Deserialize)]
pub struct HtmlPlaylist{
    pub videos:Vec<VideoHtml>,//paths of all videos, path is a unique identifier
    pub name:String,//name of playlist
}
impl VideoDB{
    fn make_thumbnails(&mut self)->Result<String,String>{
        for file in self.database.iter_mut(){
            //make thumbnail 
            if file.is_video(){
                let thumb = thumbnail::make_thumb(file.file_path.clone(),
                    self.thumb_dir.clone(),self.thumb_res.clone());
                    file.metadata=db::Metadata{thumbnail_path:thumb.path,thumbnail_name:thumb.name,
                        thumbnail_res:thumb.resolution,video_data:db::VideoData{
                        star_rating:0,
                        rating: "".to_string(),
                        description:"".to_string(),
                    }};
                
            }
        }
        return Ok("sucessfully made thumbnails".to_string());
    }
    pub fn get_vid_html_vec(&self,path_base:String,thumbnail_base:String)->Vec<VideoHtml>{
        let mut vec_out:Vec<VideoHtml>=Vec::new();
        for file in self.database.iter(){
            if file.is_video(){
                let name = file.name.clone();
                let mut url = path_base.clone();
                url.push_str(&name);

                let mut thumbnail_name=thumbnail_base.clone();
                thumbnail_name.push_str(&file.metadata.thumbnail_name.clone());
                vec_out.push(VideoHtml{name:file.name.clone(),
                    url:url.clone(),thumbnail_url:thumbnail_name,
                    html_url:url.clone(),path:file.file_path.clone()});
            }
        }
        return vec_out;
    }
    pub fn get_vid_html(&self,path_base:String,thumbnail_base:String,
            vid_name:String)->Result<VideoHtml,String>{
        for file in self.database.iter(){
            if file.name==vid_name{

            let name = file.name.clone();
            let mut url = path_base;
            url.push_str(&name);

            let mut thumbnail_name=thumbnail_base.clone();
            thumbnail_name.push_str(&file.metadata.thumbnail_name.clone());
            return Ok(VideoHtml{name:file.name.clone(),url:url.clone(),thumbnail_url:thumbnail_name,
                html_url:url,path:file.file_path.clone()});
            }
        }
        return Err("video not found".to_string());

    }
    pub fn get_vid_data(&self,vid_path:String)->Result<VideoRatingData,String>{
        let res = self.database.get_file_from_path(vid_path.clone());
        if res.is_ok(){
            let vid = res.unwrap();
            let out = VideoRatingData{star_rating:vid.metadata.video_data.star_rating,
                rating:vid.metadata.video_data.rating.clone(),
                description: vid.metadata.video_data.description.clone()};
            return Ok(out);
        }
        else{
            return Err(format!("videos.rs get_vid_data: path {} not found",vid_path));
        }
    }
    pub fn get_vid_html_from_path(&self,path_base:String,
        thumbnail_base:String,vid_path:String)->Result<VideoHtml,String>{
        let res = self.database.get_file_from_path(vid_path);
        if res.is_ok(){
            let file = res.ok().unwrap();
            let mut thumbnail_name=thumbnail_base.clone();
            let mut url = path_base.clone();
            thumbnail_name.push_str(&file.metadata.thumbnail_name);
            url.push_str(&file.name);
            return Ok(VideoHtml{name:file.name,url:url.clone(),thumbnail_url:thumbnail_name,html_url:url,path:file.file_path.clone()});

        }else{
            return Err(res.err().unwrap());
        }
    }
    pub fn add_playlist(&mut self, playlist_name:String,video_paths:Vec<String>)->Result<String,String>{
        return self.database.add_playlist(playlist_name,video_paths);
    }
    pub fn get_playlist_all(&self,path_base:String,thumbnail_base:String)->Vec<HtmlPlaylist>{
        let temp_play = self.database.get_playlist_all();
        let mut out_vec = Vec::new();
        out_vec.reserve(temp_play.len());
        for play in temp_play{
            let mut vid_vec = Vec::new();
            vid_vec.reserve(play.video_paths.len());
            for name in play.video_paths{
                let res = self.get_vid_html_from_path(path_base.clone(),thumbnail_base.clone(),name);
                if res.is_ok(){
                    vid_vec.push(res.ok().unwrap());
                }
            }
            out_vec.push(HtmlPlaylist{name:play.name,videos:vid_vec})
        }
        return out_vec;
    }
    //gets the path of a video with a certain name
    pub fn get_vid_path(&self,name:String)->Result<String,String>{
        for file in self.database.iter(){
            if file.name==name{
                return Ok(file.file_path.clone()); 
            }
        }
        return Err("video not found".to_string());
    }
    pub fn iter(&self)->std::slice::Iter<'_,db::FileData>{
        return self.database.iter();
    }
}
fn is_video(path_str: String)->bool{
    let path = Path::new(&path_str);
    let ext_opt = path.extension();
    let mut extension = "".to_string();
    if ext_opt.is_some(){
        let foo = ext_opt.unwrap();
        extension=foo.to_str().unwrap().to_string();
    }
    if path.is_file() && (extension=="m4v".to_string() || extension=="ogg".to_string() || extension=="mp4".to_string()){
        return true; 
    }else{
        return false;
    }
}
pub fn new(read_dir:String,thumb_dir:String,database_path:String,thumb_res:u32)->Result<VideoDB,String>{
    let make_db = db::new(database_path,read_dir);
    if make_db.is_ok(){
        let mut video_db=VideoDB{database:make_db.ok().unwrap(),thumb_dir:thumb_dir,thumb_res:thumb_res};
        let thumb_res = video_db.make_thumbnails();
        if thumb_res.is_ok(){
            return Ok(video_db);
        }else{
            return Err(thumb_res.err().unwrap());
        }
    }else{
        return Err(make_db.err().unwrap());
    }
}
pub fn empty()->VideoDB{
    return VideoDB{database:db::empty(),thumb_dir:"".to_string(),thumb_res:0};
}
