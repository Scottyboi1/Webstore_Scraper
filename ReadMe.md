# Webstore_Scrapper API
## Description
This is a rust program that scrapes listings from Goodwillfinds and Ebay.
## API Usage

Render link: https://webstore-scrapper.onrender.com

### Python Example

Download python library (https://pypi.org/project/Webstore-ScrapperPY/):

```
pip install Webstore-ScrapperPY
```

Python Code:

```
from Webstore_ScrapperPY import fetch_data
fetch_data('laptop')
```

### JavaScript Example
```
async function fetchData(query) {
    const response = await fetch(`https://webstore-scrapper.onrender.com/search?query=${query}`);
    
    if (response.ok) {
        const blob = await response.blob();
        const url = window.URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.style.display = 'none';
        a.href = url;
        a.download = 'output.csv';
        document.body.appendChild(a);
        a.click();
        window.URL.revokeObjectURL(url);
    } else {
        console.error('Failed to retrieve data:', response.status);
    }
}
fetchData('user input here');
```

### C/C++ Example
```
#include <curl/curl.h>
#include <iostream>
#include <fstream>

size_t WriteCallback(void* contents, size_t size, size_t nmemb, void* userp) {
    ((std::string*)userp)->append((char*)contents, size * nmemb);
    return size * nmemb;
}

void fetchData(const std::string& query) {
    CURL* curl;
    CURLcode res;
    std::string readBuffer;

    curl = curl_easy_init();
    if(curl) {
        std::string url = "https://webstore-scrapper.onrender.com/search?query=" + query;
        curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
        curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, WriteCallback);
        curl_easy_setopt(curl, CURLOPT_WRITEDATA, &readBuffer);
        res = curl_easy_perform(curl);
        curl_easy_cleanup(curl);

        if(res != CURLE_OK) {
            std::cerr << "Failed to retrieve data: " << curl_easy_strerror(res) << std::endl;
        } else {
            std::ofstream outfile("output.csv");
            outfile << readBuffer;
            outfile.close();
        }
    }
}

int main() {
    fetchData("User input here");
    return 0;
}
```

## TODO
1. Create libraries in Javascript and C/C++ (more languages may be implemented later).
2. Add parameters/filtering of the data.
3. Add time parameter. User can adjust amount of pages to scrape or the amount of time to scrape.
4. Add additional websites to scrape, ex: OfferUp or Craigslist etc...
