use std::borrow::BorrowMut;
/*
* SPDX-License-Identifier: MIT
*/
use std::cell::RefCell;
use std::rc::Rc;

use crate::rendering::vertex::VertexBuffer;
use crate::rendering::vertex::VertexDesc;
use crate::rendering::vertex::Vertices;
use gl::types::{GLenum, GLint, GLsizei, GLsizeiptr, GLuint};

use crate::gl_check;

use super::gl_program::ShaderProgram;

#[repr(u32)]
#[derive(Debug)]
#[derive(Clone, Copy)]
pub enum DrawingMode {
    Points = gl::POINTS,
    Lines = gl::LINES,
    LineLoop = gl::LINE_LOOP,
    LineStrip = gl::LINE_STRIP,
    Triangles = gl::TRIANGLES,
    TriangleStrip = gl::TRIANGLE_STRIP,
    TriangleFan = gl::TRIANGLE_FAN,
    LinesAdjacency = gl::LINES_ADJACENCY,
    LineStripAdjacency = gl::LINE_STRIP_ADJACENCY,
    TrianglesAdjacency = gl::TRIANGLES_ADJACENCY,
    TrianglesStripAdjacency = gl::TRIANGLE_STRIP_ADJACENCY,
}

pub struct GlOject {
    vao: GLuint,
    vbo: GLuint,
    ebo: GLuint,
    index_count: GLint,
    drawing_mode: DrawingMode,
    program: Rc<ShaderProgram>,
}

impl GlOject {
    pub fn new<T>(vertices: &Vertices<T>, program: Rc<ShaderProgram>) -> Self {
        let mut vao: GLuint = 0;
        let mut vbo: GLuint = 0;
        let mut ebo: GLuint = 0;
        match &vertices.buffer {
            VertexBuffer::Array(v) => {
                setup_vertex_objects(&mut vao, &mut vbo, v);
                setup_attrib_pointer(&vertices.desc, &program);
                Self {
                    vao,
                    vbo,
                    ebo,
                    index_count: 0,
                    drawing_mode: DrawingMode::Triangles,
                    program,
                }
            }
            VertexBuffer::Indexed(v, indices) => {
                let index_count = indices.len() as GLint;
                setup_vertex_objects(&mut vao, &mut vbo, v);
                setup_element_objects(&mut ebo, indices);
                setup_attrib_pointer(&vertices.desc, &program);
                Self {
                    vao,
                    vbo,
                    ebo,
                    index_count,
                    drawing_mode: DrawingMode::Triangles,
                    program,
                }
            }
        }
    }

    pub fn bind(&self) {
        unsafe {
            if self.vao > 0 {
                gl_check!(gl::BindVertexArray(self.vao));
            }
        }
    }

    pub fn draw(&self) {
        unsafe {
            self.bind();
            self.program.activate().expect("Fail to use program");
            if self.index_count > 0 {
                gl_check!(gl::DrawElements(
                    self.drawing_mode as u32,
                    self.index_count,
                    gl::UNSIGNED_INT,
                    std::ptr::null()
                ));
            } else {
                gl_check!(gl::DrawArrays(self.drawing_mode as u32, 0, 3));
            }
        }
    }

    pub fn update<T>(&mut self, vertices: VertexBuffer<T>) {
        let verts = match vertices {
            VertexBuffer::Array(verts) => verts,
            VertexBuffer::Indexed(verts, indices) => {
                self.index_count = indices.len() as GLint;
                unsafe {
                    gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo));
                    gl_check!(gl::BufferData(
                        gl::ELEMENT_ARRAY_BUFFER,
                        (indices.len() * std::mem::size_of::<GLuint>()) as GLsizeiptr,
                        indices.as_ptr() as *const _,
                        gl::STATIC_DRAW,
                    ));
                };
                verts
            }
        };
        unsafe {
            gl_check!(gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo));
            gl_check!(gl::BufferData(
                gl::ARRAY_BUFFER,
                (verts.len() * std::mem::size_of::<T>()) as GLsizeiptr,
                verts.as_ptr() as *const _,
                gl::STATIC_DRAW,
            ));
        };
    }

    pub fn set_drawing_mode(&mut self, mode: DrawingMode) {
        self.drawing_mode = mode;
    }

    pub fn drawing_mode(&self) -> DrawingMode {
        self.drawing_mode
    }
}

impl Drop for GlOject {
    fn drop(&mut self) {
        unsafe {
            if self.vbo > 0 {
                gl_check!(gl::DeleteBuffers(1, &self.vbo));
            }
            if self.ebo > 0 {
                gl_check!(gl::DeleteBuffers(1, &self.ebo));
            }
            if self.vao > 0 {
                gl_check!(gl::DeleteVertexArrays(1, &self.vao));
            }
        }
    }
}

#[inline]
fn setup_vertex_objects<T>(vao: &mut u32, vbo: &mut u32, v: &Vec<T>) {
    unsafe {
        gl_check!(gl::GenVertexArrays(1, vao));
        gl_check!(gl::BindVertexArray(*vao));
        gl_check!(gl::GenBuffers(1, vbo));
        gl_check!(gl::BindBuffer(gl::ARRAY_BUFFER, *vbo));
        gl_check!(gl::BufferData(
            gl::ARRAY_BUFFER,
            (v.len() * std::mem::size_of::<T>()) as GLsizeiptr,
            v.as_ptr() as *const _,
            gl::STATIC_DRAW,
        ));
    };
}

#[inline]
fn setup_element_objects(ebo: &mut u32, indices: &Vec<GLuint>) {
    unsafe {
        gl_check!(gl::GenBuffers(1, ebo));
        gl_check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, *ebo));
        gl_check!(gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * std::mem::size_of::<GLuint>()) as GLsizeiptr,
            indices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        ));
    };
}

#[inline]
fn setup_attrib_pointer(descs: &Vec<VertexDesc>, program: &ShaderProgram) {
    for desc in descs {
        unsafe {
            let location = program.get_attribute_location(&desc.attribute);
            gl_check!(gl::VertexAttribPointer(
                location,
                desc.size,
                gl::FLOAT,
                gl::FALSE,
                desc.stride as GLsizei,
                desc.offset as *const _
            ));
            gl_check!(gl::EnableVertexAttribArray(location));
        };
    }
}
