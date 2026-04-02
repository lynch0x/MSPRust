// dllmain.cpp : Definiuje punkt wejścia dla aplikacji DLL.

#include <windows.h>
#include <wininet.h>
#include <iostream>
#include <cstdint>
static HINTERNET hInternet, hConnect;
const DWORD_PTR context = 92348448;
const char agent[87] = "Mozilla/5.0 (Windows; U; pl-PL) AppleWebKit/533.19.4 (KHTML, like Gecko) AdobeAIR/32.0";
    extern "C" {
        // Function to free the allocated memory
        void free_response(const uint8_t* ptr) {
            delete[] ptr;  // MUST match the allocation method in NetConnection
        }
        const BYTE* NetConnection(const char* host, const char* path, BYTE* data, int32_t length, int32_t& response_length) {
            if(!hInternet){
            hInternet = InternetOpenA(agent, INTERNET_OPEN_TYPE_PRECONFIG, 0, 0, 0);
    InternetSetOptionA(hInternet, INTERNET_OPTION_USER_AGENT, (void*)agent, sizeof(agent));
            }
            if (!hConnect) {
                hConnect = InternetConnectA(hInternet, host, INTERNET_DEFAULT_HTTPS_PORT, NULL, NULL, INTERNET_SERVICE_HTTP, 0, 0);
                if (hConnect == NULL) {
                    InternetCloseHandle(hInternet);
                    return NULL;  
                }
            }
            LPCSTR headers = "x-flash-version: 32,0,0,100\r\n"
                "Content-Type: application/x-amf\r\n"
                "Accept-Encoding: gzip, deflate\r\n";
            HINTERNET hRequest = HttpOpenRequestA(hConnect, "POST", path, NULL, "app:/cache/t1.bin/[[DYNAMIC]]/2",
                NULL, INTERNET_FLAG_KEEP_CONNECTION | INTERNET_FLAG_RELOAD | INTERNET_FLAG_RESYNCHRONIZE | INTERNET_FLAG_SECURE, 0);
            if (hRequest == NULL) {
                InternetCloseHandle(hConnect);
                InternetCloseHandle(hInternet);
                return NULL; 
            }
            if (!HttpSendRequestA(hRequest, headers, strlen(headers), (void*)data, length)) {
                InternetCloseHandle(hRequest);
                return NULL;
            }
            int32_t contentLength = 0;
            DWORD dwBufferLength = sizeof(contentLength);
            BOOL querySuccess = HttpQueryInfoA(hRequest, HTTP_QUERY_CONTENT_LENGTH | HTTP_QUERY_FLAG_NUMBER, &contentLength, &dwBufferLength, NULL);
            if (!querySuccess || contentLength <= 0) {
                InternetCloseHandle(hRequest);
                return NULL;
            }
            response_length = contentLength;
            BYTE* buffer = new BYTE[response_length];
            DWORD bytesRead = 0;
            DWORD totalBytesRead = 0;
            while (InternetReadFile(hRequest, buffer + totalBytesRead, response_length - totalBytesRead, &bytesRead) && bytesRead > 0) {
                totalBytesRead += bytesRead;
                if (totalBytesRead >= response_length) {
                    break;
                }
            }
            InternetCloseHandle(hRequest);
            return buffer;
        }
    }

