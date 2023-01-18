use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGlBuffer;
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader, Window};
use js_sys::{Function, Object, Reflect, WebAssembly};
use std::cell::RefCell;
use std::rc::Rc;


fn create_vertices_from_wave_data(wave_data: &[f32]) -> Vec<f32> {
    let mut vertices = vec![];
    for i in 0..wave_data.len() - 1 {
        vertices.push(i as f32);
        vertices.push(wave_data[i]);
        vertices.push(0.0);

        vertices.push((i + 1) as f32);
        vertices.push(wave_data[i + 1]);
        vertices.push(0.0);

        vertices.push((i + 1) as f32);
        vertices.push(0.0);
        vertices.push(0.0);
    }
    vertices
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("surface-canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let context = canvas
        .get_context("webgl")?
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()?;

    const VERTEX_SHADER_SRC: &str = r#"
        attribute vec2 a_position;
        uniform vec2 u_resolution;
        uniform float u_time;
        varying vec4 v_color;
    
        void main() {
            vec2 position = a_position;
            position.y += (sin(a_position.x / u_resolution.x * 3.14 * 2.0 + u_time) + 1.0) * 24.0;
            vec2 zero_to_one = position / u_resolution;
            vec2 zero_to_two = zero_to_one * 2.0;
            vec2 clip_space = zero_to_two - 1.0;
            gl_Position = vec4(clip_space * vec2(1, -1), 0, 1);
            v_color = vec4(1.0, 0.0, 0.0, 1.0);
        }
    "#;
    
    const FRAGMENT_SHADER_SRC: &str = r#"
        precision mediump float;
        varying vec4 v_color;
    
        void main() {
            gl_FragColor = vec4(0.0, 0.0, 1.0, 0.8);
        }
    "#;

    let vertex_shader = compile_shader(&context, WebGlRenderingContext::VERTEX_SHADER, VERTEX_SHADER_SRC)?;
    let fragment_shader = compile_shader(&context, WebGlRenderingContext::FRAGMENT_SHADER, FRAGMENT_SHADER_SRC)?;

    let program = link_program(&context, &vertex_shader, &fragment_shader)?;
    context.use_program(Some(&program));
    
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut i = 0;
    let mut time = 0.0;
    *g.borrow_mut() = Some(Closure::new(move || {
        if i > 2000000 {
            let _ = f.borrow_mut().take();
            // i += 1;
            
            return;
        }
        i += 1;
        time += 0.02;
        // let time = web_sys::window().unwrap().performance() .unwrap().now() as f32 / 1000.0;
        let resolution = (canvas.width() as f32, canvas.height() as f32);
        let wave_data = generate_wave(time);
        let vertices = create_vertices_from_wave_data(&wave_data);
        render(&context, &program, &vertices, resolution, time);
        request_animation_frame(f.borrow().as_ref().unwrap());
    }));
    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

fn render(context: &WebGlRenderingContext, program: &WebGlProgram, vertices: &[f32], resolution: (f32, f32), time: f32) {
    
    let vertex_buffer = context.create_buffer().unwrap();
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
    unsafe {
        let vert_array = js_sys::Float32Array::view(&vertices);
        context.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &vert_array,
            WebGlRenderingContext::STATIC_DRAW,
        );
    }


    let a_position = context
        .get_attrib_location(&program, "a_position");

    context.enable_vertex_attrib_array(a_position as u32);
    context.vertex_attrib_pointer_with_i32(a_position as u32, 3, WebGlRenderingContext::FLOAT, false, 0, 0);

    let u_resolution = context.get_uniform_location(&program, "u_resolution").unwrap();
    context.uniform2f(Some(&u_resolution), resolution.0, resolution.1);

    let u_time = context.get_uniform_location(&program, "u_time").unwrap();
    context.uniform1f(Some(&u_time), time);

    context.clear_color(1.0, 1.0, 1.0, 1.0);
    context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
    
    context.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, (vertices.len() / 3) as i32);
}


pub fn compile_shader(
    context: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Could not create shader"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false) 
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Could not get shader info log")))
    }
}

pub fn link_program(
    context: &WebGlRenderingContext,
    vertex_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
       .create_program()
       .ok_or_else(|| String::from("Could not create program"))?;

    context.attach_shader(&program, vertex_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);
    
    if context
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
           .get_program_info_log(&program)
           .unwrap_or_else(|| String::from("Could not get program info log")))
    }

}

#[wasm_bindgen]
pub fn generate_wave(time: f32) -> Vec<f32> {
    let mut wave_data = vec![];
    for x in 0..1200 {
        let y = (x as f32 / 1200.0 * std::f32::consts::PI * 2.0 + time).sin();
        wave_data.push((y+1.0)* 24.0);
    }

    println!("time:{:?}", time);
    println!("wave_data:{:?}", wave_data);
    wave_data
}
