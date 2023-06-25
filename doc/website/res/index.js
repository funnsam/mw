window.addEventListener('DOMContentLoaded', () => {
	const a = document.getElementsByTagName("a");
	for (let i = 0; i < a.length; i++) {
		a[i].onclick = (e) => {
			window.top.location.href = e.srcElement.getAttribute("data-to");
		};
	}
})
