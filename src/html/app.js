let image_input = document.querySelector("#img-input");

let originalImage = document.querySelector("#original")
let imageContainer = document.querySelector("#img-container")


let id = 1000
function createImage(base64, label="") {

    const currentDate = new Date();
    const timestamp = currentDate.getTime();
    
    let imageLabel = document.createElement("label")
    imageLabel.for = id
    id++
    const text = label + " (" + timestamp.toString() + ")" 
    imageLabel.innerHTML = text

    let img = document.createElement("img")
    img.src = `data:image/png;base64,${base64}`
    imageContainer.append(label)
    imageContainer.append(img)
    return true;
}

image_input.addEventListener("change", async () => {
    if(image_input.files.length > 0) {
        let file = image_input.files[0]
        let imageData = new FormData()
        imageData.append("file", file);

        let reader = new FileReader();
        reader.onload = (evt) => {
            originalImage.src = evt.target.result
        }
        reader.readAsDataURL(file)
        
        let selectedAlgo = document.querySelector("#algo").value

        let sigma = document.querySelector("#sigma").value;
        let threshold = document.querySelector("#threshold").value;

        fetchImage(file, selectedAlgo, sigma, threshold)

    }
})


async function fetchImage(image, algo, sigma, threshold) {

    switch(algo) {
        case "canny": 
            let data1 = await canny(image, sigma, threshold)
            return createImage(data1.data.base64, algo)
        case "sobel":
            let data2 = await sobel(image, sigma);
            return createImage(data2.data.base64, algo)
        case "all":
            let img1 = await canny(image, sigma, threshold)
            createImage(img1.data.base64, "Canny")
            let img2 = await sobel(image, sigma)
            createImage(img2.data.base64, "Sobel")
            return true
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
        let res = await fetch("/sobel", {
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