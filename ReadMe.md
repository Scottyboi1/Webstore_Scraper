# Webstore_Scrapper API
## Description
This is a rust program that scrapes listings from Goodwillfinds and Ebay.
## API Usage
### Python Example
```
import requests
def fetch_data(query):
    response = requests.get(f'https://webstore-scrapper.onrender.com/search?query={query}')
    if response.status_code == 200:
        # Save the CSV content to a file
        with open('output.csv', 'wb') as f:
            f.write(response.content)
    else:
        print("Failed to retrieve data:", response.status_code)
fetch_data('user input here')
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