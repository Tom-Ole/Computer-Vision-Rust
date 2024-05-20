let image_input = document.querySelector("#img-input");

let img1 = document.querySelector("#img1")
let img2 = document.querySelector("#img2")

image_input.addEventListener("change", async () => {
    if(image_input.files.length > 0) {
        let file = image_input.files[0]
        let imageData = new FormData()
        imageData.append("file", file);

        let reader = new FileReader();
        reader.onload = (evt) => {
            img1.src = evt.target.result
        }
        reader.readAsDataURL(file)
        
        let res = await fetch("/loadImage", {
            method: "POST",
            headers: {
                "Content-length": length
            },
            body: file
        });

        let data = await res.json()
        if(data.data && data.data.base64) {
            img2.src = `data:image/png;base64,${data.data.base64}`;
        }

    }
})
