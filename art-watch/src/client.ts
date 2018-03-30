const ws = new WebSocket("ws://localhost:1999");
const container = document.querySelector("body")!;

ws.onopen = function () {
    console.log("opened!");

    this.onmessage = (message) => {
        const filename: string = message.data;
        console.log(filename);
        if (/\.(svg|png|jpg)$/.test(filename)) {
            const rand = Math.random();
            container.innerHTML = `<img src="${filename}?${rand}" />`;
        }
    }
};

ws.onerror = function (err: Event) {
    console.error(err);
};
