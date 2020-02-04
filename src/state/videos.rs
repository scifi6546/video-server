use serde::{Deserialize,Serialize};
use std::path::Path;
use gulkana;

mod thumbnail;
#[derive(Clone,Serialize,Deserialize)]
pub struct VideoData{
    pub star_rating:u32,//star rating (eg 5 or 4 stars)
    pub rating:String,//normal rating (eg pg, pg13)
    pub description:String,//Dexcription Of video
}
#[derive(Clone,Serialize,Deserialize)]
pub struct Metadata{
    pub thumbnail_name:String,
    pub thumbnail_path:String,
    pub thumbnail_res:u32,
    pub video_data:VideoData,
    //to add stuff
}
#[derive(Clone,Serialize,Deserialize)]
pub struct FileData{
    pub file_name: String,
    pub name: String,
    pub file_path:String,
    pub extension:String,
    pub metadata:Metadata,
}
impl FileData{
    pub fn is_video(&self)->bool{
        if self.extension=="m4v".to_string() || self.extension=="ogg".to_string() ||
            self.extension=="mp4".to_string(){
            
            return true;
        }else{
            return false;
        }
    }
}

#[derive(Clone,Serialize,Deserialize)]
pub struct VideoHtml{
    pub name: String,
    pub url: String,
    pub thumbnail_url: String,
    pub html_url:String,
    pub path:String,
    pub video_data:VideoData,
}
//used to edit video
#[derive(Clone,Serialize,Deserialize)]
pub struct VideoEditData{
    pub star_rating:u32,//star rating (eg 5 or 4 stars)
    pub rating:String,//normal rating (eg pg, pg13)
    pub description:String,//Dexcription Of video
    pub name:String,//name to change to
}
#[derive(Clone,PartialEq,Serialize,Deserialize)]
enum DirectoryTypes{
    Directory,
    Playlist,
}
#[derive(Clone)]
pub struct VideoDB{
    database:gulkana::DataStructure<String,FileData,DirectoryTypes>,
    thumb_dir:String,
    thumb_res:u32,
}
#[derive(Clone,Serialize,Deserialize)]
pub struct HtmlPlaylist{
    pub videos:Vec<VideoHtml>,//paths of all videos, path is a unique identifier
    pub name:String,//name of playlist
}
fn empty_video_rating()->VideoData{
    return VideoData{star_rating:0,rating:"".to_string(),description:"".to_string(),}; 
}
impl VideoDB{
    fn make_thumbnails(&mut self)->Result<String,String>{
        let mut keys = vec![];
        for (key,_file) in self.database.iter_data(){
            keys.push(key.clone());
        }
        for key in keys{
            //make thumbnail 
            let file_res = self.database.get(&key);
            if file_res.is_ok(){
                let mut file = file_res.ok().unwrap().clone();
            if file.is_video(){
                let thumb_res = thumbnail::make_thumb(file.file_path.clone(),
                    self.thumb_dir.clone(),self.thumb_res.clone());
                if thumb_res.is_ok(){
                    let thumb=thumb_res.unwrap();
                    file.metadata=Metadata{thumbnail_path:thumb.path,thumbnail_name:thumb.name,
                        thumbnail_res:thumb.resolution,video_data:file.metadata.video_data.clone()
                    };
                    self.database.set_data(&key,&file);
                }else{
                    return Err(thumb_res.err().unwrap());
                }
                
            }
            }
        }
        return Ok("sucessfully made thumbnails".to_string());
    }
    pub fn get_vid_html_vec(&self,path_base:String,html_path_base:String,thumbnail_base:String)->Vec<VideoHtml>{
        let mut vec_out:Vec<VideoHtml>=Vec::new();
        for (_key,file) in self.database.iter_data(){
            if file.is_video(){
                let name = file.name.clone();
                let mut file_url = path_base.clone();
                file_url.push_str(&name);
                let mut html_url = html_path_base.clone();
                html_url.push_str(&name);
                
                let video_data = VideoData{rating:file.metadata.video_data.rating.clone(),
                    star_rating:file.metadata.video_data.star_rating,
                    description:file.metadata.video_data.description.clone()};
                println!("video_description: {}",video_data.description);
                let mut thumbnail_name=thumbnail_base.clone();
                thumbnail_name.push_str(&file.metadata.thumbnail_name.clone());
                vec_out.push(VideoHtml{
                    name:file.name.clone(),
                    url:file_url.clone(),
                    thumbnail_url:thumbnail_name,
                    html_url:html_url.clone(),
                    path:file.file_path.clone(),
                    video_data:video_data,
                });
            }
        }
        return vec_out;
    }
    pub fn get_vid_html(&self,path_base:String,thumbnail_base:String,
            vid_name:String)->Result<VideoHtml,String>{
        for (_key,file) in self.database.iter_data(){
            if file.name==vid_name{

            let name = file.name.clone();
            let mut url = path_base;
            url.push_str(&name);

            let video_data = VideoData{rating:file.metadata.video_data.rating.clone(),
                    star_rating:file.metadata.video_data.star_rating,
                    description:file.metadata.video_data.description.clone()};
            let mut thumbnail_name=thumbnail_base.clone();
            thumbnail_name.push_str(&file.metadata.thumbnail_name.clone());
            return Ok(VideoHtml{name:file.name.clone(),url:url.clone(),thumbnail_url:thumbnail_name,
                html_url:url,path:file.file_path.clone(),
                video_data:video_data, 
            });
            }
        }
        return Err("video not found".to_string());

    }
    pub fn get_vid_data(&self,vid_path:String)->Result<VideoData,String>{
        let res = self.database.get(&vid_path.clone());
        if res.is_ok(){
            let vid = res.ok().unwrap();
            let out = VideoData{star_rating:vid.metadata.video_data.star_rating,
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
        let res = self.database.get(&vid_path);
        if res.is_ok(){
            let file = res.ok().unwrap();
            let mut thumbnail_name=thumbnail_base.clone();
            let mut url = path_base.clone();
            thumbnail_name.push_str(&file.metadata.thumbnail_name);
            url.push_str(&file.name);
            let video_data = VideoData{rating:file.metadata.video_data.rating.clone(),
                    star_rating:file.metadata.video_data.star_rating,
                    description:file.metadata.video_data.description.clone()};
            return Ok(VideoHtml{name:file.name.clone(),url:url.clone(),
                thumbnail_url:thumbnail_name,html_url:url,
                path:file.file_path.clone(),
                video_data:video_data, 
            });

        }else{
            return Err("Key not found".to_string());
        }
    }
    pub fn edit_video_data_path(&mut self,path:String,
            to_change_to: VideoEditData)->Result<String,String>{
        let res = self.database.get(&path);
        if res.is_ok(){
            let mut data = res.ok().unwrap().clone();
            
            data.metadata.video_data=
                VideoData{rating: to_change_to.rating,star_rating: to_change_to.star_rating,
                description:to_change_to.description};
            self.database.set_data(&path,&data);
            return Ok("success".to_string());
        }else{
            return Err("file not found".to_string());
        }
    }
    pub fn add_playlist(&mut self, playlist_name:String,video_paths:Vec<String>)->Result<String,String>{
        let res = self.database.overwrite_link(&playlist_name,&video_paths,DirectoryTypes::Playlist);
        if res.is_ok(){
            return Ok("success".to_string());
        }else{
            return Err("failed to make playlist".to_string());
        }
    }
    pub fn edit_playlist(&mut self,playlist_name:String,video_paths:Vec<String>)->Result<String,String>{
        let res = self.database.overwrite_link(&playlist_name,&video_paths,DirectoryTypes::Playlist);
        if res.is_ok(){
            return Ok("success".to_string());
        }else{
            return Err("failed to make playlist".to_string());
        }

    }
    pub fn get_playlist_all(&self,path_base:String,thumbnail_base:String)->Vec<HtmlPlaylist>{
        let mut playlist_list = vec![]; 
        for (link,linked_keys) in self.database.iter_link_type(&DirectoryTypes::Playlist){
            let mut vid_vec = vec![];
            for key in linked_keys{
            let vid_res = self.get_vid_html(path_base.clone(),key.clone(),thumbnail_base.clone());
                if vid_res.is_ok(){
                    vid_vec.push(vid_res.ok().unwrap()); 
                }
            }
            playlist_list.push(HtmlPlaylist{videos:vid_vec,name:link});
        }
        return playlist_list;
    }
    //gets the path of a video with a certain name
    pub fn get_vid_path(&self,name:String)->Result<String,String>{
        for (_key,video) in self.database.iter_data(){
            if video.name==name{
                return Ok(video.file_path.clone()); 
            }
        }
        return Err("video not found".to_string());
    }
    pub fn iter(&self)->gulkana::DataNodeIter<'_, std::string::String, FileData, DirectoryTypes>{
        return self.database.iter_data();
    }
    pub fn get_thumb_res(&self)->Result<u32,String>{
        return Ok(self.thumb_res);
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
pub fn new(_read_dir:String,thumb_dir:String,database_path:String,thumb_res:u32)->Result<VideoDB,String>{
    let make_db = gulkana::backed_datastructure(&database_path);
    let mut video_db=VideoDB{database:make_db,thumb_dir:thumb_dir,thumb_res:thumb_res};
    let thumb_res = video_db.make_thumbnails();
    if thumb_res.is_ok(){
        return Ok(video_db);
    }else{
        return Err(thumb_res.err().unwrap());
    }
}
pub fn empty()->VideoDB{
    return VideoDB{database:gulkana::new_datastructure(None),thumb_dir:"".to_string(),thumb_res:0};
}
#[cfg(test)]
mod test{
}
