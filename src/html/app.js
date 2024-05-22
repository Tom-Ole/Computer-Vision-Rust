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
        
        let selectedAlgo = document.querySelector("#algo").value

        let sigma = document.querySelector("#sigma").value;
        let threshold = document.querySelector("#threshold").value;

        let data = await fetchImage(file, selectedAlgo, sigma, threshold)

        if(data.data && data.data.base64) {
            img2.src = `data:image/png;base64,${data.data.base64}`;
        }

    }
})


async function fetchImage(image, algo, sigma, threshold) {

    switch(algo) {
        case "canny": 
            return canny(image, sigma, threshold)
        case "sobel":
            return sobel(image, sigma);
        default: 
            return null;
    }

}

async function setSigma(sigma) {
    let r1 = await fetch("/setSigma", {
        method: "POST",
        headers: {
            "Content-length": sigma.toString().length
        },
        body: sigma.toString()
    });
    return r1;
} 

async function setThreshold(threshold) {
    let r1 = await fetch("/setThreshold", {
        method: "POST",
        headers: {
            "Content-length": threshold.toString().length
        },
        body: threshold.toString()
    });
    return r1
}

async function sobel(image, sigma) {
    let r1 = await setSigma(sigma)
    if(r1.ok) {
        let res = await fetch("/canny", {
            method: "POST",
            headers: {
                "Content-length": image.length
            },
            body: image
        });
        return await res.json()
    } else {
        return null
    }
}

async function canny(image, sigma, threshold) {
    let r1 = await setSigma(sigma)
    let r2 = await setThreshold(threshold)

    if(r1.ok && r2.ok) {
        let res = await fetch("/canny", {
            method: "POST",
            headers: {
                "Content-length": image.length
            },
            body: image
        });
        
        return await res.json()
    } else {
        return null
    }
}