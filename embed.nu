let url = "https://cdn.jsdelivr.net/npm/@scalar/api-reference"
mkdir asset
http get $url | save -f asset/scalar.js
