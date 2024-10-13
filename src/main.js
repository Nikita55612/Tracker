// Импорт и экспорт функций
const { invoke } = window.__TAURI__.core;

// Экспорт функций для глобального использования
export { processSearchRequest, downloadItemByID, getItemFilesList, hideResults };
window.processSearchRequest = processSearchRequest;
window.downloadItemByID = downloadItemByID;
window.getItemFilesList = getItemFilesList;
window.hideResults = hideResults;

const alertBox = document.getElementById("alertBox");
const alertText = document.getElementById("alertText");

// Константы
const LOADING_TEXT = 'Loading...';
const FILE_NAME_REGEX = /<b>(.*?)<\/b>/g;


function errorMessage(error) {
  alertText.innerHTML = error;
  alertBox.style.display = "block";
  console.error(error);
}

/**
 * Скрывает окно результатов поиска.
 */
function hideResults() {
  document.getElementById("searchResultsWindow").style.display = "none";
}

/**
 * Извлекает имена файлов из HTML-строки.
 * @param {string} htmlString - HTML-строка для анализа.
 * @return {string[]} Массив имен файлов.
 */
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

/**
 * Обрабатывает запрос поиска.
 */
async function processSearchRequest() {
  const query = document.getElementById("searchQueryInput").value.trim();
  if (!query) return;

  try {
    const result = await invoke("search_query", { query });
    const searchResults = document.getElementById("searchResults");
    const searchResultsWindow = document.getElementById("searchResultsWindow");
    const obj = JSON.parse(result);
    searchResults.innerHTML = generateSearchResultsHTML(obj);
    searchResultsWindow.style.display = "block";
  } catch (error) {
    errorMessage(`Error processing search request: ${error}`);
  }
}

/**
 * Загружает элемент по ID.
 * @param {string} itemId - ID элемента для загрузки.
 */
async function downloadItemByID(itemId) {
  try {
    await invoke("download_item", { itemId });
  } catch (error) {
    errorMessage(`Error downloading item: ${error}`);
  }
}

/**
 * Получает список файлов для элемента.
 * @param {string} itemId - ID элемента.
 */
async function getItemFilesList(itemId) {
  const filesListElement = document.querySelector(`ul[item-id="${itemId}"] > li > ul[role="menu"]`);
  if (filesListElement && filesListElement.textContent.includes(LOADING_TEXT)) {
    try {
      const result = await invoke("get_item_files_list", { itemId });
      const filesList = extractFileNames(result);
      filesListElement.innerHTML = filesList.map(item => `<li>${item}</li>`).join('');
    } catch (error) {
      errorMessage(`Error getting item files list: ${error}`);
    }
  }
}

/**
 * Генерирует HTML для результатов поиска.
 * @param {Object[]} jsonData - Данные результатов поиска.
 * @return {string} HTML-строка результатов поиска.
 */
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
    <div class="separator"></div>`
  ).join('').replace(/\n[^\n]*$/, '');
}

// Event Listeners
document.addEventListener('DOMContentLoaded', () => {
  document.addEventListener('keydown', event => {
    if (event.key === 'Enter' && document.activeElement.id === "searchQueryInput") {
      processSearchRequest();
    }
  });
});