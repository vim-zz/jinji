use crate::filters::Banana;
use crate::functions::HttpGet;
use anyhow::Result;
use base64::{engine::general_purpose::STANDARD_NO_PAD as base64, Engine as _};
use clap::Parser;
use regex::{Captures, Regex};
use std::borrow::Cow;
use std::fs;
use std::io;
use std::io::Read;
use std::str;
use tera::{Context, Tera, Value};

mod filters;
mod functions;

// Define constants for separator patterns and encoding marks
const SEPERATOR_BEGIN_PATTERN: &str = r"---";
const SEPERATOR_END_PATTERN: &str = r"---";
const ENC_BEGIN_MAR: &str = r"_JINJI_ENC_BEGIN_MARK_";
const ENC_END_MARK: &str = r"_JINJI_ENC_END_MARK_";

/// Represents command-line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the source file. If not provided, reads from stdin
    #[arg(short, long)]
    source: Option<String>,
}

fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    // Read the source file or stdin into a string
    let input = match args.source {
        Some(file_path) => fs::read_to_string(file_path)?,
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            buffer
        }
    };

    let textfile: Vec<&str> = input.lines().collect();

    // Finding the positions of the header separator in the file
    let seperator_begin = textfile
        .iter()
        .position(|&line| line == SEPERATOR_BEGIN_PATTERN);

    let seperator_end = if let Some(index) = seperator_begin {
        textfile
            .iter()
            .skip(index + 1) // start right after last match
            .position(|&line| line == SEPERATOR_END_PATTERN)
            .map(|result| result + 1) // need to compensate the index value for the 1 we took
    } else {
        None
    };

    // Initialize Tera template engine
    let mut tera = Tera::default();

    // Context for storing and passing data to templates
    let mut context = Context::new();

    // Extract and process header if it exists
    if let (Some(begin), Some(end)) = (seperator_begin, seperator_end) {
        let header_text = textfile[begin + 1..end].join("\n");
        cyclic_render_of_the_header(&mut tera, &mut context, &header_text)?;
    }

    // Extract body text from the source file
    let body_text = if let Some(end) = seperator_end {
        &textfile[end + 1..]
    } else {
        &textfile
    };
    let body = body_text.join("\n");

    // Register custom filter and function for Tera rendering
    tera.register_filter("banana", Banana {});
    tera.register_function("http_get", HttpGet::default());

    // Render the template with the provided context
    let text = tera.render_str(&body, &context)?;

    // Output
    println!("{text}");

    Ok(())
}

// Encodes specified patterns in the input string using base64
//
// This function searches for patterns in 'source' that match the regular expression 're',
// and replaces them with their base64-encoded equivalents.
//
// Args:
//   re: Reference to a Regex that identifies the patterns to be encoded.
//   source: The input string where the patterns will be searched and encoded.
//
// Returns:
//   A 'Cow' (Copy on Write) string with the encoded patterns.
fn encode_pattern<'a>(re: &'a Regex, source: &'a str) -> Cow<'a, str> {
    re.replace_all(source, |caps: &Captures| {
        let encoded = base64.encode(&caps[1]);
        format!("{ENC_BEGIN_MAR}{encoded}{ENC_END_MARK}")
    })
}

// Decodes base64-encoded patterns in the input string
//
// This function searches for encoded patterns in 'source' that match the regular expression 're',
// and decodes them from base64 back to their original form.
//
// Args:
//   re: Reference to a Regex that identifies the encoded patterns.
//   source: The input string with encoded patterns to be decoded.
//
// Returns:
//   A 'Cow' (Copy on Write) string with the decoded patterns.
fn decode_pattern<'a>(re: &'a Regex, source: &'a str) -> Cow<'a, str> {
    re.replace_all(source, |caps: &Captures| {
        // decode the middle, leaving the 2 markers out
        let buf = base64.decode(&caps[2]).unwrap();
        let decoded = str::from_utf8(&buf).unwrap();
        format!("{decoded}")
    })
}

// Decodes JSON values that contain encoded strings
//
// This recursive function iterates over a JSON object, looking for strings that match
// a specific encoded pattern, and decodes them.
//
// Args:
//   json: A mutable reference to a JSON 'Value' object to be decoded.
//
// Returns:
//   A 'Result' indicating the success or failure of the operation.
fn decode_json(json: &mut Value) -> Result<()> {
    for (_key, value) in json.as_object_mut().unwrap() {
        match value {
            Value::String(text) => {
                // use lazy evak here, to capture the minimal pattern
                let re = Regex::new(&format!(r"({ENC_BEGIN_MAR})(.*?)({ENC_END_MARK})"))?;
                *value = Value::String(decode_pattern(&re, text).to_string());
            }
            Value::Object(_) => {
                decode_json(&mut *value)?;
            }
            _ => {}
        }
    }

    Ok(())
}

/// Recursively renders the header of a source file using Tera templating engine.
///
/// This function takes the header text of a source file and processes it through the Tera
/// templating engine. During this process, it handles any template expressions and YAML data
/// present in the header. It recursively renders the header to support scenarios where
/// the rendering process might update the data used in the template, thus requiring multiple
/// rendering passes.
///
/// # Arguments
///
/// * `tera`: A mutable reference to a `Tera` instance, the templating engine used for rendering.
/// * `context`: A mutable reference to a `Context`, which holds the data to be used in the template rendering.
/// * `header_text`: A string slice containing the header text to be rendered.
///
/// # Behavior
///
/// The function initially encodes expressions in the header that could potentially break YAML syntax.
/// It then proceeds to decode the encoded expressions after loading the header as a YAML object.
/// This YAML object is then flattened and its contents are inserted into the context.
/// The function checks if the rendering result differs from the input header text.
/// If it does, it recursively calls itself with the updated header text to ensure
/// that all dynamic content is fully rendered.
///
/// # Returns
///
/// * `Result<()>`: A result type which, on success, returns an empty tuple (`()`), indicating
///   successful rendering. On failure, it returns an error type provided by the `anyhow` crate.
///
fn cyclic_render_of_the_header(
    tera: &mut Tera,
    context: &mut Context,
    header_text: &str,
) -> Result<()> {
    // Load the header as YAML

    // Encode all expressions as they might brake YAML syntax and prevent loading
    let re = Regex::new(r"(\{\{.*?\}\})")?;
    let header_encoded = encode_pattern(&re, header_text);
    // load YAML
    let mut header_json: Value = serde_yaml::from_str(&header_encoded)?;
    // Decode all expressions
    decode_json(&mut header_json)?;

    // add json object into context at the root level (flat)
    for (key, value) in header_json.as_object().unwrap() {
        context.insert(key, value);
    }

    // Render the context to support cyclic assigment
    let result = tera.render_str(header_text, context)?;
    if result != header_text {
        cyclic_render_of_the_header(tera, context, &result)?;
    }

    Ok(())
}
