# jquery-typewriter-plugin

A customizable typing animation with jquery.

![Demo](https://github.com/ZaphodElevated/typewriter-plugin/blob/master/assets/typewriter_demo.gif?raw=true)

# Usage

Install the jquery-typewriter plugin with npm with the following command.

```
npm i jquery-typewriter
```

Add the jquery cdn in your html file along with the path to the js and css files.

```html
<script src="https://ajax.googleapis.com/ajax/libs/jquery/3.5.1/jquery.min.js"></script>

<script src="node_modules/jquery-typewriter/dist/js/jquery.typewriter.min.js"></script>

<link
  rel="stylesheet"
  href="node_modules/jquery-typewriter/dist/css/cursor.css"
/>
```

In your main js file, you can use the typewrite function on any element.

The function requires an object with which you can costumize the typewriter.

```js
$('h1').typeWrite({
  speed: 30,
  repeat: false,
  cursor: true,
  color: 'blue',
})
```
