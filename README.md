# Jinji

## Description

Jinji is a Rust-based tool for dynamic content generation, utilizing Tera templating and REGEX for pattern matching. It automates and streamlines complex text file creation, ideal for system configuration, and data transformation, offering a customizable, scalable text manipulation solution.

## Features

- Tera template rendering: Process and render examples dynamically.
- Header extraction and rendering: Supports cyclic rendering of headers in source files.
- Markdown to HTML conversion: Converts markdown text to HTML format.
- Custom Tera filters and functions: Includes implementations like the `Banana` filter and `HttpGet` function.

## Getting Started

### Installation

```
cargo install jinji
```` 

### Usage

To run the project:

```bash
cargo run -- --source <path_to_your_source_file>
```

#### Example

`jinji --source examples/concept.md`
OR
`cat examples/concept.md | jinji`

Source at: 
```yaml+jinja
---
vars:
    a1: yahoo

calls:
    url1: https://google.com 
    url2: https://{{ vars.a1 }}.com
    url3: {{ calls.url1 | upper }}xxx{{ calls.url2 }}:{{ 2 * 8 }}

fruits: ["Apple", "Banana", "Cherry"]
---

# General form

How much us 8 x 6 ? it's {{ 8 * 6 }}

And that's {{ vars.a1 | upper }}!

At: {{ calls.url1 | banana(count=3) }}

Get: {{ http_get(url=calls.url1) | get(key="headers") |  get(key="date") }}

Cyclic: {{ calls.url3 }}

{{ "2019-09-19T13:18:48.731Z" | date(timezone="America/New_York") }}
{{ 1648252203 | date(format="%A %-d %B", timezone="Europe/Paris", locale="fr_FR") }}

{{ 3 * 1.20 * 22 + 3 * 2.00 * 22 }}

{% for fruit in fruits %}
  <li>{{ fruit }}</li>
{% endfor %}
```

Result:

```
# General form

How much us 8 x 6 ? it's 48

And that's YAHOO!

At: 🍌🍌🍌https://google.com🍌🍌🍌

Get: Fri, 22 Mar 2024 09:04:03 GMT

Cyclic: HTTPS://GOOGLE.COMxxxhttps://yahoo.com:16

2019-09-19
Saturday 26 March

211.2


  <li>Apple</li>

  <li>Banana</li>

  <li>Cherry</li>
```

## License

Distributed under the MIT License. See `LICENSE` for more information.
