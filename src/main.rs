use yew::prelude::*;
use std::collections::HashMap;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;

use gloo::file::callbacks::FileReader;
use gloo::file::File;
use gloo_console::log;

use web_sys::{DragEvent, MouseEvent, Event, FileList, HtmlInputElement};
use wasm_bindgen::JsValue;

use yew::html::TargetCast;
use yew::{html, Callback, Component, Context, Html};

use image::{GrayImage, *};
use svg::node::element::Group;
use truchet::{image::Image, vec2::Vec2, svg::node::element::SVG, to_svg::ToSVG};

struct FileDetails {
    name: String,
    file_type: String,
    data: Vec<u8>
}

struct ImageAdapter {
    image: GrayImage
}

impl ImageAdapter {
    fn new(image: GrayImage) -> Self { Self { image } }
}

impl Image for ImageAdapter {
    fn size(&self) -> Vec2<usize> {
        return Vec2::new(self.image.dimensions().0 as usize, self.image.dimensions().1 as usize);
    }

    fn get_pixel_brightness(&self, pos: Vec2<usize>) -> f32 {
        return self.image.get_pixel(pos.x() as u32, pos.y() as u32).0[0] as f32 / 255.0;
    }
}

pub enum Msg {
    Loaded(String, String, Vec<u8>),
    Files(Vec<File>),
    GenerateButtonClicked(bool),
    TileDropdownClicked(bool)
}

pub struct App {
    readers: HashMap<String, FileReader>,
    files: Vec<FileDetails>,
    tile_dropdown_is_open: bool,
    tile_dropdown_opened_classes: String,
    tile_dropdown_closed_classes: String
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            readers: HashMap::default(),
            files: Vec::default(),
            tile_dropdown_is_open: false,
            tile_dropdown_opened_classes: classes!("rounded-md","bg-white","focus:outline-none").to_string(),
            tile_dropdown_closed_classes: classes!("rounded-md","bg-white","focus:outline-none","hidden").to_string(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Loaded(file_name, file_type, data) => {
                self.files.push(FileDetails {
                    data,
                    file_type,
                    name: file_name.clone(),
                });
                self.readers.remove(&file_name);
                true
            }
            Msg::Files(files) => {
                for file in files.into_iter() {
                    let file_name = file.name();
                    let file_type = file.raw_mime_type();

                    let task = {
                        let link = ctx.link().clone();
                        let file_name = file_name.clone();

                        gloo::file::callbacks::read_as_bytes(&file, move |res| {
                            link.send_message(Msg::Loaded(
                                file_name,
                                file_type,
                                res.expect("failed to read file"),
                            ))
                        })
                    };
                    self.readers.insert(file_name, task);
                }
                true
            }
            Msg::GenerateButtonClicked(bool) => {
                if self.files.len() == 0 {
                    return false
                } 
                if self.files.len() > 0 {
                    let image = image::load_from_memory(&self.files[0].data);
                    let grayscale = ImageAdapter::new(image.expect("Should be valid image").into_luma8());
                    let truchet = truchet::truchet_image::generate(&grayscale, truchet::generator::circles(Vec2::new(6, 6)));
                    let svg = truchet.to_svg_node();
                    let g = Group::new()
                        .set("transform", "scale(10 10)")
                        .add(svg);
                    let svg_doc = SVG::new()
                        .add(g)
                        .set("height", "500px")
                        .set("width", "500px");
                    let mut data = vec![];
                    svg::write(&mut data, &svg_doc).unwrap();
                    let data = String::from_utf8(data).unwrap();
                    let document = web_sys::window()
                        .unwrap()
                        .document()
                        .unwrap();
                    let output_area = document.get_element_by_id("truchet-output-area").expect("Output area should exist");
                    output_area.set_inner_html(&data);
                }
                true
            }
            Msg::TileDropdownClicked(bool) => {
                let document = web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap();
                let dropdown = document.get_element_by_id("tile-dropdown-wrapper")
                    .expect("Should exist");
                let mut class_name = dropdown.class_name();
                match self.tile_dropdown_is_open {
                    true => {
                        dropdown.set_class_name(&self.tile_dropdown_opened_classes);
                        log!("Hello", JsValue::from(dropdown.id()));
                    },
                    false => {
                        dropdown.set_class_name(&self.tile_dropdown_closed_classes);
                        log!("Hello", JsValue::from(dropdown.id()));
                    }
                }
                self.tile_dropdown_is_open = !self.tile_dropdown_is_open;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let tile_dropdown_on_click = ctx.link().callback(|event: MouseEvent| {
            event.stop_propagation();
            Msg::TileDropdownClicked(true)
        });

        let generate_button_on_click = ctx.link().callback(|event: MouseEvent| {
            event.stop_propagation();
            Msg::GenerateButtonClicked(true)
        });

        html! {
            <div id="wrapper-wrapper" class={ classes!("flex","content-center","w-screen","h-screen","p-20") }>
                <div id="wrapper" class={ classes!("bg-slate-500","flex","flex-row","space-x-10","outline","outline-grey","w-screen","justify-around") }>
                    <div id="image-upload" class={ classes!("flex","flex-col","space-y-2","outline-dashed","outline-white") }>
                        <p id="image-upload-title">{ "Upload Your Image to Process!" }</p>
                        <label for="file-upload">
                            <div
                                id="drop-container"
                                ondrop={ctx.link().callback(|event: DragEvent| {
                                    event.prevent_default();
                                    let files = event.data_transfer().unwrap().files();
                                    Self::upload_files(files)
                                })}
                                ondragover={Callback::from(|event: DragEvent| {
                                    event.prevent_default();
                                })}
                                ondragenter={Callback::from(|event: DragEvent| {
                                    event.prevent_default();
                                })}
                            >
                                <i class="fa fa-cloud-upload"></i>
                                <p>{"Drop your images here or click to select"}</p>
                            </div>
                        </label>
                        <input
                            id="file-upload"
                            type="file"
                            accept="image/*"
                            multiple={true}
                            onchange={ctx.link().callback(move |e: Event| {
                                let input: HtmlInputElement = e.target_unchecked_into();
                                Self::upload_files(input.files())
                            })}
                        />
                        <div id="preview-area">
                            { for self.files.iter().map(Self::view_file) }
                        </div>
                    </div>

                    <div id="settings" class={ classes!("flex","flex-col","outline-dashed","outline-white") }>
                        <p 
                            id="settings-title"
                        >
                            { "Settings!" }
                        </p>
                        <ul 
                            id="settings-wrapper"
                        >
                            <li 
                                id="tile-selection" 
                                class={ classes!("flex","flex-col","") }
                            >
                                <div id="">
                                    <div id="tile-preview">
                                    </div>
                                </div>
                                <div 
                                    id="tile-selection" 
                                    class={ classes!("flex","flex-row","whitespace-nowrap") }
                                >
                                    <button 
                                        id="tile-dropdown" 
                                        type="button" 
                                        class={ classes!("inline-flex","w-full","rounded-md")}
                                        onclick={tile_dropdown_on_click}
                                    >
                                        <svg 
                                            id="dropdown-arrow"
                                            class={ classes!("-mr-1","h-5","w-5","text-gray-400") } 
                                            viewBox="0 0 20 20" 
                                            fill="currentColor" 
                                            aria-hidden="true"
                                        >
                                            <path 
                                                fill-rule="evenodd" 
                                                d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" 
                                                clip-rule="evenodd" 
                                            />
                                        </svg>
                                    </button>
                                    <label for="tile-dropdown">
                                        {"Tile selection"}
                                    </label>     
                                </div>
                                <div 
                                    id="tile-dropdown-wrapper"
                                    class={ classes!("rounded-md","bg-white","focus:outline-none") } 
                                    role="menu" 
                                    aria-orientation="vertical" 
                                    aria-labelledby="menu-button" 
                                    tabindex="-1"
                                >
                                    <div 
                                        id="tile-dropdown-menu"
                                        class="py-1" 
                                        role="none"
                                    >
                                        <a href="#" class="text-gray-700 block px-4 py-2 text-sm" role="menuitem" tabindex="-1" id="menu-item-0">{"Circle"}</a>
                                        <a href="#" class="text-gray-700 block px-4 py-2 text-sm" role="menuitem" tabindex="-1" id="menu-item-1">{"Triangle"}</a>
                                    </div>
                                </div>
                                <form method="POST" action="#" role="none" class={ classes!("my-20") }>
                                    <button 
                                        type="submit" 
                                        class={ classes!("bg-white","text-gray-700","block","w-full","px-4","py-2","text-left","text-sm") } 
                                        role="menuitem" 
                                        tabindex="-1" 
                                        id="menu-item-3"
                                    >
                                        {"Apply"}
                                    </button>
                                </form>
                            </li>
                            <li>{""}</li>
                            <li>{""}</li>
                        </ul>
                        <label for="generate">
                            <button 
                                class={ classes!("bg-pink-500","hover:bg-emerald-500","text-white","font-bold","rounded") }
                                onclick={generate_button_on_click}
                            >
                                <p>{"Generate Truchet!"}</p>
                            </button>
                        </label>
                    </div>

                    <div id="truchet-result" class={ classes!("flex","outline-dashed","outline-white") }>
                        <p id="truchet-result-title">{ "Truchet!" }</p>
                        <div id="truchet-output-area">
                            
                        </div>
                    </div>
                </div>
            </div> 
        }
    }
}

impl App {
    fn view_file(file: &FileDetails) -> Html {
        html! {
            <div class="preview-tile">
                <p class="preview-name">{ format!("{}", file.name) }</p>
                <div class="preview-media">
                    if file.file_type.contains("image") {
                        <img src={format!("data:{};base64,{}", file.file_type, STANDARD.encode(&file.data))} />
                    } else if file.file_type.contains("video") {
                        <video controls={true}>
                            <source src={format!("data:{};base64,{}", file.file_type, STANDARD.encode(&file.data))} type={file.file_type.clone()}/>
                        </video>
                    }
                </div>
            </div>
        }
    }

    fn upload_files(files: Option<FileList>) -> Msg {
        let mut result = Vec::new();

        if let Some(files) = files {
            let files = js_sys::try_iter(&files)
                .unwrap()
                .unwrap()
                .map(|v| web_sys::File::from(v.unwrap()))
                .map(File::from);
            result.extend(files);
        }
        Msg::Files(result)
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}