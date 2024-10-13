

function processSearchRequest_() {
    processSearchRequest()
}

function downloadItem(el) {
    const itemId = el.getAttribute("item-id");
    downloadItemByID(itemId);
}

function hideResults_() {
    hideResults()
}

function getItemFilesList_(el) {
    const itemId = el.getAttribute("item-id");
    getItemFilesList(itemId);
}

function closeAlert() {
    const alertBox = document.getElementById("alertBox");
    alertBox.style.display = "none";
}