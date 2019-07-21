from pyramid.view import view_config
from pyramid.view import view_defaults

from pyramid.response import Response
from pyramid.response import FileResponse

from pyramid.security import (remember,forget)

from pyramid.httpexceptions import HTTPFound
from .videos import VideoArr
from .config import Config
from .users import Users
import json
@view_defaults(renderer='home.jinja2')
class StateMgr:
    def __init__(self):
        self.Config=Config()
        if(self.Config.getConfig()!={}):
            self.Videos=VideoArr(self.Config.getConfig()['videos'])
            self.isSetup=True
            self.users=Users(self.Config.getConfig()['users'])
        else:
            self.Videos=VideoArr(None)
            self.isSetup=False
            self.users=Users([])
    def _write(self):
        to_write={}
        to_write["users"]=self.users.getConfig()
        to_write["videos"]=self.Videos.getConfig()
        self.Config.write(to_write)
        self.isSetup=True
    def configure(self,username,password,video_dir):
        self.addUser(username,password)
        self.Videos.setVideoPath(video_dir)
        self._write()
    def addUser(self,username,password):
        self.users.addUser(username,password)
        self._write()
    def addUserAuth(self,user_adding,username,password):
        if self.isPriviliged(user_adding):
            return self.addUser(username,password)
    def rmUserAuth(self,user_adding,username):
        if self.isPriviliged(user_adding) and len(self.users.users)>1 and user_adding!=username:
            print("removed user")
            self.users.rmUser(username)
        else:
            print("did not remove user")
            return {"status":"failed adding user"}
        self._write()

    def checkPasswd(self,username,password):
        return self.users.checkPassword(username,password)
    def getVideos(self,username):
        if(self.isPriviliged(username)):
            return self.Videos.getVideos()
        else:
            return []
    def getUserInfo(self,username):
        print("priviliged?")
        if self.isPriviliged(username):
            print("user_info: ")
            print(self.users.getUserInfo())
            return self.users.getUserInfo()
        else:
            print("user " + username +"not priviliged")
            return {"status":"not priviliged"}
    def isPriviliged(self,username):
        return self.users.isPriviliged(username)
    def getVideoByURL(self,username,url):
        if(self.isPriviliged(username)):
            return self.Videos.getVideoByURL(url)
        else:
            return {"status":"not privliged"}
        return
    def getVideoByName(self,username,name):
        if(self.isPriviliged(username)):
            return self.Videos.getVideoByName(name)
        return

    #gets configuration menue in dictionary form
    #[{"name":"NAME OF field","description":"description","items":['values','value2']}]
    def getConfigMenu(self):
        
        return [{"name":"users"}]
    def changeVideoPath(self,username,path):
        if(self.isPriviliged(username)):
            self.Videos.setVideoPath(path) 
            temp_cfg=self.Config.getConfig()
            temp_cfg["video_path"]=path
            self.Config.write(temp_cfg)
    def makePlaylist(self,username,video_names,playlist_name):
        if(self.isPriviliged(username)):
            self.Videos.makePlaylist(video_names,playlist_name)
            self._write()
            return {"status":"success"}
        else:
            return {"status":"not priviliged"}
    def getPlaylists(self,username):
        if(self.isPriviliged(username)):
            return self.Videos.getPlaylistsWeb()





state=StateMgr()
class MainView:
    def __init__(self,request):
        self.request=request
        self.logged_in = request.authenticated_userid
    @property
    def counter(self):
        session = self.request.session
        if 'counter' in session:
            session['counter'] +=1
        else:
            session['counter']=1
        print(session['counter'])
        counter=session['counter']
        return counter
    @view_config(route_name='index',renderer='home.jinja2')
    def index(self):
        print(self.logged_in)
        if(self.isSetup()==False):
            print("redirecting to config screen")
            return HTTPFound(self.request.route_url("setup"))
        if self.logged_in is None:
            
            return HTTPFound(self.request.route_url("login"))

        videoArr_temp=[]
        for i in state.getVideos(self.logged_in):
            videoArr_temp.append({"url":i.getUrl(),"name":i.getName(),
                "html_url":self.request.route_url("video_html",url=i.getUrl()),"thumb_url":self.request.route_url("thumbnails",name=i.getName())});

        print(self.request.route_url("logout"))
        return {"LOGOUT_URL":self.request.route_url("logout"),"videos":videoArr_temp,"CONFIG_URL":self.request.route_url("config"),}
    @view_config(route_name='login',renderer='login.jinja2')
    def login(self):
        request=self.request
        login_url=request.route_url('login')
        referrer=request.url
        if(referrer==login_url):
            referrer='/'
        came_from = request.params.get('came_from',referrer)
        message=''
        username=''
        password=''
        if 'form.submitted' in request.params:
            username = request.params['username']
            password = request.params['password']
            if(state.checkPasswd(username,password)):
                print("Password Sucessfull")
                headers=remember(request,username)
                return HTTPFound(location=came_from,headers=headers)
        message = 'Failed Login'
        return {"ACTION_URL":request.application_url+"/login"}
    @view_config(route_name='logout')
    def logout(self):
        request=self.request
        headers=forget(request)
        url=request.route_url('index')
        return HTTPFound(location=url,headers=headers)
    @view_config(route_name='setup',renderer="setup.jinja2")
    def setup(self):
        if(self.isSetup()==True):
            return HTTPFound(location=self.request.route_url("index"))
        else:
            if 'form.submitted' in self.request.params:
                temp_config={"video_path":self.request.params["video_path"],
                    "users":[]}
                username=self.request.params["username"]
                password=self.request.params["password"]
                video_path=self.request.params["video_path"]
                state.configure(username,password,video_path)
                return HTTPFound(location=self.request.route_url("index"))
            return {}
    @view_config(route_name="playlists",renderer="playlists.jinja2")
    def playlistMasterHtml(self):
        plays = state.getPlaylists(self.logged_in)
        for i in range(0,len(plays)):
            plays[i]["thumbnail"]=self.request.route_url("thumbnails",
                    name=plays[i]["thumbnail"]) 
        print("plays")
        print(plays)

        return {"playlists":plays}
    def isSetup(self):
        return state.isSetup
    @view_config(route_name="video") 
    def video(self):
        print("handled video")
        url = self.request.matchdict['url']
        print(url)
        temp_vid = state.getVideoByURL(self.logged_in,url)
        print(temp_vid)
        return FileResponse(temp_vid.getFilePath())
    @view_config(route_name="video_html",renderer='video.jinja2')
    def video_html(self):
        url_t = self.request.matchdict['url']
        temp_url=self.request.route_url("video",url=url_t)
        print("temp_url: ")
        print(temp_url)
        if(state.isPriviliged(self.logged_in)):
            return {"video":{"url":temp_url}}
        return {}
    @view_config(route_name="config",renderer="config.jinja2")
    def configMenue(self):
        print("printed config")
        print(state.getUserInfo(self.logged_in))
        return {"users":state.getUserInfo(self.logged_in)}
    @view_config(route_name="user_api",renderer="json")
    def adduserAPI(self):
        print(self.request.body)
        data=json.loads(self.request.body.decode('utf8'))
        if(data["action"]=="add user"):
            username=data["username"]
            password=data["password"]
            state.addUserAuth(self.logged_in,username,password)
        if(data["action"]=="remove user"):
            username=data["username"]
            print("removed users")
            state.rmUserAuth(self.logged_in,username)
    @view_config(route_name="video_path_api",renderer="json")
    def videoPathAPI(self):
        data=json.loads(self.request.body.decode('utf8'))
        if(data["action"]=="change data path"):
            path=data["path"]
            state.changeVideoPath(self.logged_in,path)
        return {} 
    @view_config(route_name="thumbnails")
    def thumbnails(self):
        vid = state.getVideoByName(self.logged_in,
                self.request.matchdict['name'])
        return FileResponse(vid.getThumb())
    
    @view_config(route_name="playlist_get",renderer="json")
    def playlist_get(self):
        if state.isPriviliged(self.logged_in):
            playlists=state.getPlaylists(self.logged_in)
            for i in range(0,len(playlists)):
                for j in range(0,len(playlists[i]["videos"])):
                    playlists[i]["videos"][j]["url"]=\
                        self.request.route_url("video",url=
                        playlists[i]["videos"][j]["url"])
                    print("\n\nplaylists: ")
                    print(playlists[i]["videos"][j]["name"])
                    playlists[i]["videos"][j]["thumbnail"]=\
                        self.request.route_url("thumbnails",name=
                        playlists[i]["videos"][j]["name"])
            print(self.request)
            print("playlists: "+ str(playlists))
            return {"playlists":playlists}
    @view_config(route_name="playlist_post",renderer="json")
    def playlist_post(self):
        if state.isPriviliged(self.logged_in):
            data=json.loads(self.request.body.decode('utf8'))
            if(data["action"]=="make_playlist"):
                videos=data["videos"]
                name=data["playlist_name"]
                state.makePlaylist(self.logged_in,videos,name)


