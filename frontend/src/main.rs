use yew::prelude::*;

use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use anyhow::Error;
use chrono::{TimeZone, Utc};
use yew::format::{Json};
use serde_json::json;

use yew::services::storage::{Area, StorageService};
const KEY: &'static str = "yew.noteapp.self";

mod structs;
use structs::{Note, NoteVector};

enum Msg {
    SyncComplete(Result<NoteVector, Error>),
    SyncFailed,
    
    NewContent(String),
    NewTitle(String),
    NewTags(String),
    EditContent(String),
    EditTitle(String),
    
    Submit,
    Delete(usize),
    Edit(usize),
    Save(usize),
}

struct Model {
    link: ComponentLink<Self>,
    ft: Option<FetchTask>,
    state: State,
    storage: StorageService,
}

pub struct State {
    link: ComponentLink<Model>,
    local_notes: NoteVector,
    url: String,

    new_title: String,
    new_content: String,
    new_tags: String,
    edit_title: String,
    edit_content: String,
    is_edited: Option<usize>,

    message: String,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        // Local storage initialization
        let storage = StorageService::new(Area::Local).expect("storage was disabled by the user");
        let local_notes = {
            if let Json(Ok(restored_model)) = storage.restore(KEY) {
                restored_model
            } else {
                NoteVector::new()
            }
        };

        let mut state = State {
            link: link.clone(),
            local_notes, 
            url: "http://127.0.0.1:8000".into(),

            new_title: "".into(),
            new_content: "".into(),
            new_tags: "".into(),
            edit_title: "".into(),
            edit_content: "".into(),
            is_edited: None,

            message: "Wassup".into(),
        };

        let task = state.sync_local_with_server();
        Self {
            link,
            ft: Some(task),
            state,
            storage,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SyncComplete(body) => {
                self.state.local_notes = body.map(|data| data).ok().unwrap();
            }
            Msg::SyncFailed => {
                self.state.message = "Sync failed".into();
            }

            Msg::NewTitle(new_value) => {
                self.state.new_title = new_value;
            }
            Msg::NewContent(new_value) => {
                self.state.new_content = new_value;
            }
            Msg::NewTags(new_value) => {
                // let vec: Vec<String> = new_value.split(',').map(|s| s.to_string()).collect();
                self.state.new_tags = new_value;
            }
            Msg::EditTitle(new_value) => {
                self.state.edit_title = new_value;
            }
            Msg::EditContent(new_value) => {
                self.state.edit_content = new_value;
            }
            Msg::Submit => {
                // If fields are empty, don't post, else post locally
                if !(self.state.new_title == "" && self.state.new_content == "") {
                    self.state.post_note_locally();

                    // Try to sync notes
                    let task = self.state.sync_local_with_server();
                    self.ft = Some(task);
                }
            }
            Msg::Delete(index) => {
                // If deleting note that is being edited, its index must be cleared
                if let Some(edited_index) = self.state.is_edited {
                    if edited_index == index {
                        self.state.is_edited = None
                    }
                }

                // If note is not yet synced with database, just remove it locally, else mark note
                // for deletion when syncing
                if self.state.local_notes[index].id == -1 {
                    self.state.local_notes.remove(index);
                }
                else {
                    self.state.local_notes[index].method = "DELETE".to_string();

                    // Try to sync notes
                    let task = self.state.sync_local_with_server();
                    self.ft = Some(task);
                }
            }
            Msg::Edit(index) => {
                // Get note with given id
                let note = &mut self.state.local_notes[index];
                
                // Toggle view for editing
                match self.state.is_edited {
                    None => {
                        self.state.is_edited = Some(index);
                        self.state.edit_title = note.title.clone();
                        self.state.edit_content = note.content.clone();
                    }
                    Some(_) => self.state.is_edited = None,
                }
            }
            Msg::Save(index) => {
                // Get note with given id
                let note = &mut self.state.local_notes[index];
                
                // Change data only if note changed
                if note.title != self.state.edit_title || note.content != self.state.edit_content {
                    // Only mark for update notes, that are already in database
                    if note.method != "CREATE" {
                        note.method = "UPDATE".to_string();
                    }

                    // Update note locally
                    let timestamp: i64 = (js_sys::Date::now() / 1000.0) as i64;
                    note.changed = timestamp;
                    note.title = self.state.edit_title.clone();
                    note.content = self.state.edit_content.clone();

                    // Try to sync
                    let task = self.state.sync_local_with_server();
                    self.ft = Some(task);
                }
                
                // Stop editing in any case
                self.state.is_edited = None;
            }
        }

        // Always save changed to browser local storage
        self.storage.store(KEY, Json(&self.state.local_notes));

        // Render again everytime
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <h1>{"notes"}</h1>
                <div class="submitArea">{ self.view_submit_area() }</div>
                <div class="mainContent">{ self.view_notes() }</div>
                <div class="result">{ &self.state.message }</div>
            </div>
        }
    }
}

impl Model {
    fn view_note_buttons(&self, editing: bool, index: usize) -> Html {
        let (onclick, text) = match editing {
            true => (self.link.callback(move |_| Msg::Save(index)), "Save"),
            false => (self.link.callback(move |_| Msg::Edit(index)), "Edit"),
        };

        html! {
            <div class="noteButtons">
                <button onclick=onclick>{ text }</button>
                <button onclick=self.link.callback(move |_| Msg::Delete(index))>{ "Delete" }</button>
            </div>
        }
    }

    fn edit_mode(&self, index: usize) -> Html {
        html! {
            <div class="innerNote">
                <textarea rows=1
                    value=self.state.edit_title.clone()
                    oninput=self.link.callback(|e: InputData| Msg::EditTitle(e.value))
                    placeholder="Title">
                </textarea>
                <textarea rows=5
                    value=self.state.edit_content.clone()
                    oninput=self.link.callback(|e: InputData| Msg::EditContent(e.value))
                    placeholder="Content">
                </textarea>
                <br />
                { self.view_note_buttons(true, index)}
            </div>
        }
    }

    fn view_mode(&self, index: usize, note: &Note) -> Html {
        // let created = Utc.timestamp(note.created, 0);
        let changed = Utc.timestamp(note.changed, 0);

        html! {
            <div class="innerNote">
                // <div>{ "Index: " }{ index }</div>
                // <div>{ "DB id: " }{ note.id }</div>
                // <div class="noteCreated">{ "Created: " }{ created }</div>
                <div class="noteChanged">{ "Changed: " }{ changed }</div>
                <div class="noteTitle">
                    <b>{ &note.title }</b>
                </div>
                <div class="noteContent">
                    { &note.content }
                </div>
                <div class="noteTags">
                    { &note.tags.join(" ~ ") }
                </div>
                { self.view_note_buttons(false, index)}
            </div>
        }
    }

    fn view_note(&self, index: usize, note: &Note) -> Html {
        html! {
            <div class="note">
                {
                    match self.state.is_edited {
                        None => self.view_mode(index, note),
                        Some(edited_index) => {
                            if edited_index == index {
                                self.edit_mode(index)
                            }
                            else {
                                self.view_mode(index, note)
                            }
                        }
                    }
                }
            </div>
        }
    }

    fn view_notes(&self) -> Html {
        let notes = self.state.local_notes
            .iter()
            .enumerate()
            .filter(|(_, note)| note.method != "DELETE") // Don't show notes marked for removal
            .map(|(index, note)| self.view_note(index, note));
        html! {
            <>
                { for notes }
            </>
        }
    }

    fn view_submit_area(&self) -> Html {
        html!(
            <div class="innerSubmitArea">
                <textarea rows=1
                    value=self.state.new_title.clone()
                    oninput=self.link.callback(|e: InputData| Msg::NewTitle(e.value))
                    placeholder="Title">
                </textarea>
                <textarea rows=5
                    value=self.state.new_content.clone()
                    oninput=self.link.callback(|e: InputData| Msg::NewContent(e.value))
                    placeholder="Content">
                </textarea>
                <textarea rows=1
                    value=self.state.new_tags.clone()
                    oninput=self.link.callback(|e: InputData| Msg::NewTags(e.value))
                    placeholder="Tags">
                </textarea>
                <div class="submitButton">
                    <button onclick=self.link.callback(|_| Msg::Submit)>{ "Submit" }</button>
                </div>
            </div>
        )
    }
}

impl State {
    fn post_note_locally(&mut self) {
        let timestamp: i64 = (js_sys::Date::now() / 1000.0) as i64;

        // Prepare note
        let note = Note {
            method: "CREATE".to_string(),
            id: -1, // to recognize note is not yet in database
            created: timestamp,
            changed: timestamp,
            title: self.new_title.clone(),
            content: self.new_content.clone(),
            tags: self.new_tags.split(",").map(|s| s.to_string()).collect(),
        };
        
        // Add note to beginning of vector
        let first = vec![note];
        let new_notevec = [first, self.local_notes.clone()].concat();
        self.local_notes = new_notevec;
        
        // Empty fields for next use
        self.new_title = "".to_string();
        self.new_content = "".to_string();
    }

    // fn get_server_notes(&mut self) -> FetchTask {
    //     let url = format!("{}", self.url);
    //     let get_request = Request::get(url)
    //         .body(Nothing)
    //         .expect("Failed to build request.");
    //     let callback = self.link.callback(
    //         |response: Response<Json<Result<NoteVector, Error>>>| {
    //             let (meta, Json(body)) = response.into_parts();
    //             if meta.status.is_success() {
    //                 return Msg::SyncComplete(body);
    //             }
    //             Msg::SyncFailed
    //         },
    //     );
    //     FetchService::fetch(get_request, callback).unwrap()
    // }

    fn sync_local_with_server(&mut self) -> FetchTask {
        let serialized = json!{self.local_notes.clone()};

        let url = format!("{}", self.url);
        let post_request = Request::post(url)
            .header("Content-Type", "application/json")
            .body(Json(&serialized))
            .expect("Failed to build request.");

        let callback = self
            .link
            .callback(|response: Response<Json<Result<NoteVector, Error>>>| {
                // If sync JSON is posted successfully, get synced notes from response
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    Msg::SyncComplete(body)
                } else {
                    Msg::SyncFailed
                }
            });
        FetchService::fetch(post_request, callback).unwrap()
    }
}

fn main() {
    yew::start_app::<Model>();
}