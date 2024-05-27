let image_input = document.querySelector("#img-input");

let originalImage = document.querySelector("#original")
let imageContainer = document.querySelector("#img-container")
let original_src;

let id = 1000
function createImage(base64, label="", container) {

    let d = new Date();
    let time = d.toLocaleTimeString();
    
    let imageLabel = document.createElement("label")
    imageLabel.for = id
    id++
    imageLabel.innerHTML = label + " (" + time + "): "

    let img = document.createElement("img")
    img.src = `data:image/png;base64,${base64}`
    container.append(imageLabel)
    container.append(img)
    return true;
}

image_input.addEventListener("change", async () => {
    if(image_input.files.length > 0) {

        let imageBox = document.createElement("div")

        let file = image_input.files[0]
        let imageData = new FormData()
        imageData.append("file", file);

        let reader = new FileReader();
        reader.onload = (evt) => {
            originalImage.src = evt.target.result;
            let d = new Date();
            let time = d.toLocaleTimeString()
    
            let imageLabel = document.createElement("label")
            imageLabel.for = id
            id++
            imageLabel.innerText = "Original " + "(" + time + "):"
            let copy_current = document.querySelector("#original").cloneNode(true);
            copy_current.id = id
            copy_current.src = evt.target.result
            imageBox.append(imageLabel);
            imageBox.append(copy_current);
        }
        reader.readAsDataURL(file)

        
        let selectedAlgo = document.querySelector("#algo").value

        let sigma = document.querySelector("#sigma").value;
        let threshold = document.querySelector("#threshold").value;

        await fetchImage(file, selectedAlgo, sigma, threshold, imageBox)
        imageContainer.prepend(imageBox)

    }
})


async function fetchImage(image, algo, sigma, threshold, container) {

    switch(algo) {
        case "canny": 
            let data1 = await canny(image, sigma, threshold)
            return createImage(data1.data.base64, algo, container)
        case "sobel":
            let data2 = await sobel(image, sigma);
            return createImage(data2.data.base64, algo, container)
        case "harris":
            let data3 = await harris(image);
            return createImage(data3.data.base64, algo, container)
        case "shi":
            let data4 = await shi(image, threshold);
            return createImage(data4.data.base64, algo, container)
        case "all":
            let data = await all(image, sigma, threshold)
            createImage(data.data.canny, "Canny", container)
            createImage(data.data.sobel, "Sobel", container)
            createImage(data.data.harris, "Harris", container)
            createImage(data.data.shi, "Shi", container)
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

async function harris(image) {
    let res = await fetch("/harris", {
        method: "POST",
        headers: {
            "Content-length": image.length
        },
        body: image
    })

    return await res.json();
}

async function shi(image, threshold) {
    let res1 = await setThreshold(threshold)
    if(res1.ok) {
        let res = await fetch("/shi", {
            method: "POST",
            headers: {
                "Content-length": image.length
            },
            body: image
        })
        return await res.json();
    } else {
        return null;
    }

}

async function all(image, sigma, threshold) {
    let r1 = await setSigma(sigma)
    let r2 = await setThreshold(threshold)

    if(r1.ok && r2.ok) {
        let res = await fetch("/all", {
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