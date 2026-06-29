let url = "https://cdn.jsdelivr.net/npm/@scalar/api-reference"
mkdir src/asset
http get $url | save -f src/asset/scalar.js
