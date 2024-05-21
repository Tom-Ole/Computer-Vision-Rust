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

        // let res = await fetch("/loadImage", {
        //     method: "POST",
        //     headers: {
        //         "Content-length": file.length
        //     },
        //     body: file
        // });

        if(data.data && data.data.base64) {
            img2.src = `data:image/png;base64,${data.data.base64}`;
        }

    }
})


async function fetchImage(image, algo, sigma, threshold) {

    switch(algo) {
        case "canny": 
            return canny(image, sigma, threshold)
        default: 
            return null;
    }

}

async function canny(image, sigma, threshold) {
    let r1 = await fetch("/setSigma", {
        method: "POST",
        headers: {
            "Content-length": sigma.toString().length
        },
        body: sigma.toString()
    });
    let r2 = await fetch("/setThreshold", {
        method: "POST",
        headers: {
            "Content-length": threshold.toString().length
        },
        body: threshold.toString()
    });

    if(r1.ok && r2.ok) {

        let res = await fetch("/loadImage", {
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
    
function toBytes(number) {
    if (!Number.isSafeInteger(number)) {
      throw new Error("Number is out of range");
    }
  
    const size = number === 0 ? 0 : byteLength(number);
    const bytes = new Uint8ClampedArray(size);
    let x = number;
    for (let i = (size - 1); i >= 0; i--) {
      const rightByte = x & 0xff;
      bytes[i] = rightByte;
      x = Math.floor(x / 0x100);
    }
  
    return bytes.buffer;
  }
  