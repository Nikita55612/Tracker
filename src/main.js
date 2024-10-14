// Импорт и экспорт функций
const { invoke } = window.__TAURI__.core;


window.processSearchRequest = processSearchRequest;
window.closeAlert = closeAlert;
window.downloadItemByID = downloadItemByID;
window.getItemFilesList = getItemFilesList;
window.hideResults = hideResults;

const alertBox = document.getElementById("alertBox");
const alertText = document.getElementById("alertText");

const LOADING_TEXT = 'Loading...';
const FILE_NAME_REGEX = /<b>(.*?)<\/b>/g;

let searchPageNum;


function errorMessage(error) {
  alertText.innerHTML = error;
  alertBox.style.display = "block";
  console.error(error);
}

function closeAlert() {
  alertBox.style.display = "none";
}

function hideResults() {
  document.getElementById("searchResultsWindow").style.display = "none";
}

function extractFileNames(htmlString) {
  const fileNames = [];
  let match;
  while ((match = FILE_NAME_REGEX.exec(htmlString)) !== null) {
    const fileName = match[1].trim();
    if (/\.\w+$/.test(fileName)) {
      fileNames.push(fileName);
    }
  }
  return fileNames;
}

async function processSearchRequest(page) {
  searchPageNum = page;
  const query = document.getElementById("searchQueryInput").value.trim();
  if (!query) {
    errorMessage("Please enter a search query.");
    return;
  }
  try {
    const result = await invoke("search_query", { query, page });
    const searchResults = document.getElementById("searchResults");
    const searchResultsWindow = document.getElementById("searchResultsWindow");
    const obj = JSON.parse(result);
    if (obj.length === 1 && obj[0].id === null) {
      errorMessage("Nothing was found for your query...");
      return;
    }
    const resultsHTML = generateSearchResultsHTML(obj);
    if (page > 0) {
      searchResults.innerHTML += resultsHTML;
      const nextResults = document.getElementsByClassName("next-search-results")[page - 1];
      if (nextResults) { nextResults.innerHTML = `<h3>Page №${page + 1}</h3>`; }
    } else { searchResults.innerHTML = resultsHTML; }
    searchResultsWindow.style.display = "block";
  } catch (error) {
    errorMessage(`Error processing search request:<br>${error}`);
  }
}

async function downloadItemByID(itemId) {
  try {
    await invoke("download_item", { itemId });
  } catch (error) {
    errorMessage(`Error downloading item<br>${error}`);
  }
}

async function getItemFilesList(itemId) {
  const filesListElement = document.querySelector(`ul[item-id="${itemId}"] > li > ul[role="menu"]`);
  if (filesListElement && filesListElement.textContent.includes(LOADING_TEXT)) {
    try {
      const result = await invoke("get_item_files_list", { itemId });
      const filesList = extractFileNames(result);
      filesListElement.innerHTML = filesList.map(item => `<li>${item}</li>`).join('');
    } catch (error) {
      errorMessage(`Error getting item files list<br>${error}`);
    }
  }
}

function generateSearchResultsHTML(jsonData) {
  return jsonData.map(item => `
    <section class="search-results-item">
      <span class="item-title">${item.title}</span>
      <table class="item-details">
        <thead>
          <tr>
            <th>Topic</th>
            <th>Author</th>
            <th>Size</th>
            <th>Downloads</th>
            <th>Date</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>${item.topic}</td>
            <td>${item.author}</td>
            <td>${item.size}</td>
            <td>${item.downloads}</td>
            <td>${item.date}</td>
          </tr>
        </tbody>
      </table>
      <div class="item-actions">
        <ul class="files-list-btn" onclick="getItemFilesList('${item.id}')" item-id="${item.id}" role="menu-bar">
          <li role="menu-item" tabindex="0" aria-haspopup="true">
            <span class="custom-underline">Files list</span>
            <ul role="menu" class="item-files-list">
              <li>${LOADING_TEXT}</li>
            </ul>
          </li>
        </ul>
        <button class="btn" onclick="downloadItemByID('${item.id}')">Download</button>
      </div>
    </section>
    <div class="separator"></div>
    `
  ).join('').concat("\n", 
    `
    <div class="next-search-results">
      <button class="btn" onclick="processSearchRequest(${searchPageNum + 1})">Next page</button>
    </div>
    `
  );
}

document.addEventListener('DOMContentLoaded', () => {
  invoke('init_config')
    .then((_) => {})
    .catch((error) => {
      errorMessage(`Config initialization error<br>${error}<br>
        Add the <a href="https://github.com/Nikita55612/Tracker/blob/main/src-tauri/config.json" target="_blank">configuration file</a> to the program directory.`)
    });
  document.addEventListener('keydown', event => {
    if (event.key === 'Enter' && document.activeElement.id === "searchQueryInput") {
      processSearchRequest(0);
    }
  });
});