// JavaScript functions that should be callable from Rust

function createElement(tagName) {
    return document.createElement(tagName);
}

function setAttribute(element, name, value) {
    element.setAttribute(name, value);
}

function addEventListener(element, eventType, callback) {
    element.addEventListener(eventType, callback);
}

function fetch(url, options) {
    return window.fetch(url, options);
}

function log(message) {
    console.log(message);
}
