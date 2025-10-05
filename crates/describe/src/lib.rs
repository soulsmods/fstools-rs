//! Structured describe output helpers for fstools.

#[cfg(test)]
extern crate self as fstools_describe;

pub use serde;
pub mod derive {
    pub use fstools_describe_derive::Describe;
}

use byte_unit::Byte;
use num_format::{Locale, ToFormattedString};
use owo_colors::{
    colors::{BrightBlack, BrightBlue, Cyan, Green},
    OwoColorize,
};
use serde::Serialize;
use supports_color::Stream;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DescribeError {
    #[error("failed to format describe output")]
    Io(#[from] std::io::Error),
    #[error("failed to serialize describe output")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, DescribeError>;

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

#[derive(Debug)]
pub enum FieldDescription {
    Attribute(DescribedAttribute),
    Children(Vec<Description>),
    Skip,
}

pub struct FieldVisitor {
    key: &'static str,
}

impl FieldVisitor {
    pub fn new(key: &'static str) -> Self {
        Self { key }
    }

    pub fn key(&self) -> &'static str {
        self.key
    }

    pub fn visit<T>(&self, value: &T) -> Result<FieldDescription>
    where
        T: VisitField,
    {
        value.visit_field(self)
    }
}

/// Helper trait implemented for values that can be rendered by the describe visitor.
pub trait VisitField {
    fn visit_field(&self, visitor: &FieldVisitor) -> Result<FieldDescription>;
}

fn attribute(visitor: &FieldVisitor, value: String) -> FieldDescription {
    FieldDescription::Attribute(DescribedAttribute::new(visitor.key(), value))
}

pub fn format_byte_size(value: u64) -> String {
    let byte = Byte::from_bytes(u128::from(value));
    let adjusted = byte.get_appropriate_unit(true);
    adjusted.format(1)
}

macro_rules! impl_display_field {
    ($($ty:ty),* $(,)?) => {
        $(
            impl VisitField for $ty {
                fn visit_field(&self, visitor: &FieldVisitor) -> Result<FieldDescription> {
                    Ok(attribute(visitor, format!("{self}")))
                }
            }
        )*
    };
}

impl_display_field!(String, str, bool, f32, f64);
macro_rules! impl_integer_field {
    ($($ty:ty),* $(,)?) => {
        $(
            impl VisitField for $ty {
                fn visit_field(&self, visitor: &FieldVisitor) -> Result<FieldDescription> {
                    let formatted = ToFormattedString::to_formatted_string(self, &Locale::en);
                    Ok(attribute(visitor, formatted))
                }
            }
        )*
    };
}

impl_integer_field!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

impl<T> VisitField for [T]
where
    T: Describe,
{
    fn visit_field(&self, _visitor: &FieldVisitor) -> Result<FieldDescription> {
        let mut children = Vec::with_capacity(self.len());
        for value in self {
            children.push(value.describe()?);
        }

        Ok(FieldDescription::Children(children))
    }
}

impl<T> VisitField for Vec<T>
where
    T: Describe,
{
    fn visit_field(&self, _visitor: &FieldVisitor) -> Result<FieldDescription> {
        let mut children = Vec::with_capacity(self.len());
        for value in self {
            children.push(value.describe()?);
        }

        Ok(FieldDescription::Children(children))
    }
}

impl<T> VisitField for Option<T>
where
    T: VisitField,
{
    fn visit_field(&self, visitor: &FieldVisitor) -> Result<FieldDescription> {
        match self {
            Some(value) => value.visit_field(visitor),
            None => Ok(FieldDescription::Skip),
        }
    }
}

impl<T> VisitField for T
where
    T: Describe,
{
    fn visit_field(&self, _visitor: &FieldVisitor) -> Result<FieldDescription> {
        Ok(FieldDescription::Children(vec![self.describe()?]))
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
    options: PlainFormatterOptions,
    colors_enabled: bool,
}

impl<'a> PlainFormatter<'a> {
    pub fn new(writer: &'a mut dyn std::io::Write) -> Self {
        Self::with_options(writer, PlainFormatterOptions::default())
    }

    pub fn with_options(
        writer: &'a mut dyn std::io::Write,
        options: PlainFormatterOptions,
    ) -> Self {
        let colors_enabled = resolve_color_choice(options.color_choice);
        Self {
            writer,
            options,
            colors_enabled,
        }
    }

    pub fn with_color_choice(mut self, choice: ColorChoice) -> Self {
        self.options.color_choice = choice;
        self.colors_enabled = resolve_color_choice(choice);
        self
    }

    pub fn write(&mut self, node: &Description) -> std::io::Result<()> {
        self.write_node(node, "", true, true)
    }

    fn write_node(
        &mut self,
        node: &Description,
        prefix: &str,
        is_last: bool,
        is_root: bool,
    ) -> std::io::Result<()> {
        let branch = if is_root {
            ""
        } else if is_last {
            "└── "
        } else {
            "├── "
        };

        let mut child_prefix = String::new();
        if !is_root {
            child_prefix.push_str(prefix);
            child_prefix.push_str(if is_last { "    " } else { "│   " });
        }

        self.write_node_line(prefix, branch, node.name)?;

        let next_prefix = if is_root { String::new() } else { child_prefix };

        if node.attributes.is_empty() && node.children.is_empty() {
            return Ok(());
        }

        let mut entries = Vec::with_capacity(node.attributes.len() + node.children.len());
        for attribute in &node.attributes {
            entries.push(LineEntry::Attribute(attribute));
        }
        for child in &node.children {
            entries.push(LineEntry::Child(child));
        }

        let total = entries.len();
        for (index, entry) in entries.into_iter().enumerate() {
            let entry_is_last = index + 1 == total;
            match entry {
                LineEntry::Attribute(attribute) => {
                    self.write_attribute(next_prefix.as_str(), entry_is_last, attribute)?;
                }
                LineEntry::Child(child) => {
                    self.write_node(child, next_prefix.as_str(), entry_is_last, false)?;
                }
            }
        }

        Ok(())
    }

    fn write_node_line(&mut self, prefix: &str, branch: &str, name: &str) -> std::io::Result<()> {
        self.write_prefix(prefix, branch)?;
        if self.colors_enabled {
            writeln!(self.writer, "{}", name.fg::<BrightBlue>().bold())
        } else {
            writeln!(self.writer, "{name}")
        }
    }

    fn write_attribute(
        &mut self,
        prefix: &str,
        is_last: bool,
        attribute: &DescribedAttribute,
    ) -> std::io::Result<()> {
        let branch = if is_last { "└── " } else { "├── " };
        self.write_prefix(prefix, branch)?;

        if self.colors_enabled {
            write!(self.writer, "{}: ", attribute.key.fg::<Cyan>().bold())?;
            writeln!(self.writer, "{}", attribute.value.as_str().fg::<Green>())
        } else {
            writeln!(self.writer, "{}: {}", attribute.key, attribute.value)
        }
    }

    fn write_prefix(&mut self, prefix: &str, branch: &str) -> std::io::Result<()> {
        if prefix.is_empty() && branch.is_empty() {
            return Ok(());
        }

        let connectors = format!("{prefix}{branch}");
        if self.colors_enabled {
            write!(self.writer, "{}", connectors.fg::<BrightBlack>())
        } else {
            write!(self.writer, "{connectors}")
        }
    }
}

fn resolve_color_choice(choice: ColorChoice) -> bool {
    match choice {
        ColorChoice::Auto => {
            supports_color::on(Stream::Stdout).is_some_and(|level| level.has_basic)
        }
        ColorChoice::Never => false,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PlainFormatterOptions {
    color_choice: ColorChoice,
}

impl PlainFormatterOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_color_choice(mut self, choice: ColorChoice) -> Self {
        self.color_choice = choice;
        self
    }

    pub fn with_colors(mut self, enabled: bool) -> Self {
        self.color_choice = if enabled {
            ColorChoice::Auto
        } else {
            ColorChoice::Never
        };
        self
    }
}

impl Default for PlainFormatterOptions {
    fn default() -> Self {
        Self {
            color_choice: ColorChoice::Auto,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ColorChoice {
    Auto,
    Never,
}

#[derive(Debug, Clone, Copy)]
#[derive(Default)]
pub struct PrintOptions {
    pub plain: PlainFormatterOptions,
}


impl PrintOptions {
    pub fn with_plain(mut self, options: PlainFormatterOptions) -> Self {
        self.plain = options;
        self
    }
}

enum LineEntry<'a> {
    Attribute(&'a DescribedAttribute),
    Child(&'a Description),
}

pub fn render<T>(value: &T, format: OutputFormat) -> Result<String>
where
    T: Describe + ?Sized,
{
    let mut output = vec![];
    print(value, format, &mut output)?;

    Ok(String::from_utf8(output).expect("should be infallible?"))
}
/// Print a node to the [`writer``] according to the requested [`OutputFormat`].
pub fn print<T>(value: &T, format: OutputFormat, writer: impl std::io::Write) -> Result<()>
where
    T: Describe + ?Sized,
{
    print_with_options(value, format, writer, PrintOptions::default())
}

pub fn print_with_options<T>(
    value: &T,
    format: OutputFormat,
    mut writer: impl std::io::Write,
    options: PrintOptions,
) -> Result<()>
where
    T: Describe + ?Sized,
{
    match format {
        OutputFormat::Plain => {
            let node = value.describe()?;
            PlainFormatter::with_options(&mut writer, options.plain).write(&node)?;
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
        version: u32,
        files: Vec<File>,
    }

    #[derive(Debug, Describe, Serialize)]
    #[describe(name = "File")]
    struct File {
        name: String,
        entries: Vec<FileEntry>,
        #[allow(dead_code)]
        #[describe(skip)]
        #[serde(skip)]
        internal_note: Option<String>,
    }

    #[derive(Debug, Describe, Serialize)]
    #[describe(name = "entry")]
    struct FileEntry {
        path: String,
    }

    #[derive(Debug, Describe, Serialize)]
    #[describe(name = "auto node")]
    struct Parent {
        title: String,
        details: Option<String>,
        child: Option<Child>,
    }

    #[derive(Debug, Describe, Serialize)]
    #[describe(name = "auto child")]
    struct Child {
        value: u32,
    }

    #[derive(Debug, Describe, Serialize)]
    #[describe(name = "stats")]
    struct Stats {
        total: u64,
        average: i32,
    }

    #[derive(Debug, Describe, Serialize)]
    #[describe(name = "formatted")]
    struct Formatted {
        #[describe(format = "0x{:X}")]
        version: u32,
        #[describe(bytes)]
        compressed_size: u64,
        #[describe(format = "addr_{:x}")]
        offset: usize,
    }

    #[test]
    fn renders_file_list_tree() -> Result<()> {
        let list = FileList {
            version: 12_345,
            files: vec![
                File {
                    name: "a.bnd".into(),
                    entries: vec![FileEntry {
                        path: "chr/c0000.anibnd".into(),
                    }],
                    internal_note: Some("first".into()),
                },
                File {
                    name: "b.bnd".into(),
                    entries: vec![FileEntry {
                        path: "map/m10/m10_00_00_00.msb".into(),
                    }],
                    internal_note: None,
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

    #[test]
    fn infers_optional_attribute_and_child() -> Result<()> {
        let populated = Parent {
            title: "root".into(),
            details: Some("with child".into()),
            child: Some(Child { value: 42_000 }),
        };

        let populated_description = populated.describe()?;
        assert_eq!(populated_description.name, "auto node");
        assert_eq!(populated_description.attributes.len(), 2);
        assert_eq!(populated_description.attributes[0].key, "title");
        assert_eq!(populated_description.attributes[0].value, "root");
        assert_eq!(populated_description.attributes[1].key, "details");
        assert_eq!(populated_description.attributes[1].value, "with child");
        assert_eq!(populated_description.children.len(), 1);
        assert_eq!(populated_description.children[0].name, "auto child");
        assert_eq!(populated_description.children[0].attributes.len(), 1);
        assert_eq!(populated_description.children[0].attributes[0].key, "value");
        assert_eq!(
            populated_description.children[0].attributes[0].value,
            "42,000"
        );

        let empty = Parent {
            title: "root".into(),
            details: None,
            child: None,
        };

        let empty_description = empty.describe()?;
        assert_eq!(empty_description.attributes.len(), 1);
        assert_eq!(empty_description.attributes[0].key, "title");
        assert_eq!(empty_description.children.len(), 0);

        Ok(())
    }

    #[test]
    fn formats_numbers_with_commas() -> Result<()> {
        let stats = Stats {
            total: 1_234_567,
            average: 98_765,
        };

        let description = stats.describe()?;
        assert_eq!(description.attributes.len(), 2);
        assert_eq!(description.attributes[0].key, "total");
        assert_eq!(description.attributes[0].value, "1,234,567");
        assert_eq!(description.attributes[1].key, "average");
        assert_eq!(description.attributes[1].value, "98,765");

        Ok(())
    }

    #[test]
    fn applies_attribute_formatting() -> Result<()> {
        let formatted = Formatted {
            version: 0x12ab,
            compressed_size: 1536,
            offset: 0x42,
        };

        let description = formatted.describe()?;
        assert_eq!(description.attributes.len(), 3);
        assert_eq!(description.attributes[0].key, "version");
        assert_eq!(description.attributes[0].value, "0x12AB");
        assert_eq!(description.attributes[1].key, "compressed_size");
        assert_eq!(description.attributes[1].value, "1.5 KiB");
        assert_eq!(description.attributes[2].key, "offset");
        assert_eq!(description.attributes[2].value, "addr_42");

        Ok(())
    }
}
