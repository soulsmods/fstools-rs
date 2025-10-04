//! Structured describe output helpers for fstools.

#[cfg(test)]
extern crate self as fstools_describe;

pub use serde;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DescribeError {
    #[error("failed to format describe output")]
    Io(#[from] std::io::Error),
    #[error("failed to serialize describe output")]
    Serialization(#[from] serde_json::Error),
}

/// A result alias for describe operations.
pub type Result<T> = std::result::Result<T, DescribeError>;

/// Represents a single attribute on an [`OutputNode`].
#[derive(Debug, Clone, Serialize)]
pub struct DescribedAttribute {
    pub key: &'static str,
    pub value: String,
}

impl DescribedAttribute {
    pub fn new(key: &'static str, value: impl Into<String>) -> Self {
        Self {
            key,
            value: value.into(),
        }
    }
}

/// Represents a structured node used for describe output.
#[derive(Debug, Clone, Serialize)]
pub struct Description {
    pub name: &'static str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<DescribedAttribute>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Description>,
}

impl Description {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            attributes: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn with_attributes(mut self, attributes: Vec<DescribedAttribute>) -> Self {
        self.attributes = attributes;
        self
    }

    pub fn with_children(mut self, children: Vec<Description>) -> Self {
        self.children = children;
        self
    }

    pub fn push_attribute(&mut self, attribute: DescribedAttribute) {
        self.attributes.push(attribute);
    }

    pub fn push_child(&mut self, child: Description) {
        self.children.push(child);
    }
}

/// Trait implemented by types that can be rendered as structured describe output.
pub trait Describe: Serialize {
    fn describe(&self) -> Result<Description>;
}

impl<T> Describe for &T
where
    T: Describe + ?Sized,
{
    fn describe(&self) -> Result<Description> {
        (**self).describe()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Plain,
    Json,
}

pub struct PlainFormatter<'a> {
    writer: &'a mut dyn std::io::Write,
    indent: usize,
}

impl<'a> PlainFormatter<'a> {
    pub fn new(writer: &'a mut dyn std::io::Write) -> Self {
        Self { writer, indent: 2 }
    }

    pub fn with_indent(mut self, indent: usize) -> Self {
        self.indent = indent.max(1);
        self
    }

    pub fn write(&mut self, node: &Description) -> std::io::Result<()> {
        self.write_node(node, 0)
    }

    fn write_node(&mut self, node: &Description, depth: usize) -> std::io::Result<()> {
        let padding = " ".repeat(self.indent * depth);
        writeln!(self.writer, "{}{}", padding, node.name)?;

        for attribute in &node.attributes {
            writeln!(
                self.writer,
                "{}{}: {}",
                " ".repeat(self.indent * (depth + 1)),
                attribute.key,
                attribute.value
            )?;
        }

        for child in &node.children {
            self.write_node(child, depth + 1)?;
        }

        Ok(())
    }
}

/// Render a node as a [`String`] according to the requested [`OutputFormat`].
pub fn print<T>(value: &T, format: OutputFormat, mut writer: impl std::io::Write) -> Result<()>
where
    T: Describe + ?Sized,
{
    match format {
        OutputFormat::Plain => {
            let node = value.describe()?;
            PlainFormatter::new(&mut writer).write(&node)?;
            Ok(())
        }
        OutputFormat::Json => Ok(serde_json::to_writer(writer, value)?),
    }
}

#[cfg(test)]
mod tests {
    use fstools_describe_derive::Describe;
    use insta::{assert_json_snapshot, assert_snapshot};

    use super::*;

    #[derive(Debug, Describe, Serialize)]
    #[describe(name = "File List")]
    struct FileList {
        #[describe(attribute)]
        version: u32,
        #[describe(children)]
        files: Vec<File>,
    }

    #[derive(Debug, Describe, Serialize)]
    #[describe(name = "File")]
    struct File {
        #[describe(attribute)]
        name: String,
        #[describe(children)]
        entries: Vec<FileEntry>,
    }

    #[derive(Debug, Describe, Serialize)]
    #[describe(name = "entry")]
    struct FileEntry {
        #[describe(attribute)]
        path: String,
    }

    #[test]
    fn renders_file_list_tree() -> Result<()> {
        let list = FileList {
            version: 3,
            files: vec![
                File {
                    name: "a.bnd".into(),
                    entries: vec![FileEntry {
                        path: "chr/c0000.anibnd".into(),
                    }],
                },
                File {
                    name: "b.bnd".into(),
                    entries: vec![FileEntry {
                        path: "map/m10/m10_00_00_00.msb".into(),
                    }],
                },
            ],
        };

        let mut plain_out = vec![];
        print(&list, OutputFormat::Plain, &mut plain_out).expect("rendered");
        let plain = String::from_utf8(plain_out).unwrap();
        assert_snapshot!("file_list_plain", plain);

        let mut json_out = vec![];
        print(&list, OutputFormat::Json, &mut json_out).expect("json");
        let root: serde_json::Value = serde_json::from_reader(&json_out[..]).expect("valid json");
        assert_json_snapshot!("file_list_json", root);

        Ok(())
    }
}
