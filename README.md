# Workers-rs Link Shortener Demo
Super basic Cloudflare worker link shortener

Yes, yet another link shortener. **It's not meant for actually use in production*** but more like a demo to get started with Cloudflare workers and D1. (https://developers.cloudflare.com/d1/)

In this demo, you will find some basic examples on how you can integrate D1 in your Worker project.  
To get started first create a D1 Database in your Cloudflare account, after that update the Cargo.toml with your credentials and just run `wrangler dev`

\* It includes no authentication for adding and removing data to and from your D1 database! Please do use it as is for any other purpose than testing.
