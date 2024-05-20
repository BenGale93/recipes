pub const BASE_TEMPLATE: &str = r##"
<!DOCTYPE html>
<head>
<script src="https://unpkg.com/htmx.org@1.9.2"></script>
</head>

<body style='padding: 30px;'>
    <div class="topnav">
    <a class="active" href="/">Home</a>
    <a href="/new">New</a>
    <a href="/roast">Roast</a>
    </div>
    {% block content %}
    {% endblock %}
</body>
"##;

pub const HOME_TEMPLATE: &str = r##"
{% extends "base" %}
{% block content %}
<div>
    <ul>
    {% for recipe in recipes %}
        <li><a href="#{{recipe.anchor}}">{{recipe.name}}</a></li>
    {% endfor %}
    </ul>
</div>
{% for recipe in recipes %}
    <div id="{{recipe.anchor}}">
        <h1>{{recipe.name}}</h1>
        <ul>
            {% for item in recipe.ingredients %}
            <li>{{ item }}</li>
            {% endfor %}
        </ul>
        <p style='white-space: pre-line;'>
            {{recipe.recipe}}
        </p>
    </div>
{% endfor %}
{% endblock %}
"##;

pub const NEW_TEMPLATE: &str = r##"
{% extends "base" %}
{% block content %}
<form hx-post="/new_recipe" hx-target="#response">
    <label>Name:</label>
    <br>
    <input type="text" name="name">
    <br>
    <label>Ingredients: </label>
    <br>
    <textarea name="ingredients"></textarea>
    <br>
    <label>Recipe: </label>
    <br>
    <textarea name="recipe"></textarea>
<input type="submit" value="Submit">
</form>

<div id="response"></div>
{% endblock %}
"##;

pub const ROAST_TEMPLATE: &str = r##"
{% extends "base" %}
{% block content %}
<form hx-post="/roast" hx-target="#response" hx-trigger="load, click">
    <label>End time:</label>
    <br>
    <input type="text" name="end" value="{{end}}">
<input type="submit" value="Submit">
</form>

<div id="response"></div>
{% endblock %}
"##;

pub const STEPS_TEMPLATE: &str = r##"
<table>
{% for (step, time) in steps %}
    <tr><td>{{step}}</td><td>{{time}}</td></tr>
{% endfor %}
</table>
"##;
