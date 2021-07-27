(function() {
	"use strict";

	function toggleTodo(event) {
		let id = event.target.parentElement.id;
		const options = {
			method: "POST"
		};
		fetch("/complete/?id=" + id, options);
	}

	window.addEventListener("load", function() {
		let elements = document.getElementsByClassName("todo-checkbox");
		for (let el of elements) {
			el.addEventListener("click", toggleTodo);
		}
	});
})();