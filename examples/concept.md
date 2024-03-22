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