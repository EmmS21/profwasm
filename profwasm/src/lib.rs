use spin_sdk::{http::{IntoResponse, Request, Response}, http_component, llm};

#[http_component]
fn handle_profwasm(_req: Request) -> anyhow::Result<impl IntoResponse> {
    let model = llm::InferencingModel::Llama2Chat;
    let css = include_str!("../static/style.css");

    let sections = vec![
        ("Containers and VMs in Cloud Computing", include_str!("../static/one.txt")),
        ("What is WebAssembly?", include_str!("../static/two.txt")),
        ("WebAssembly usage in other languages", include_str!("../static/three.txt")),
        ("How WebAssembly can be used to build applications in the cloud", include_str!("../static/four.txt")),
    ];

    let mut content_html = String::new();
    for (index, (header, content)) in sections.iter().enumerate() {
        let display_style = if index == 0 { "" } else { "style='display:none;'" };
        content_html.push_str(&format!("<h3 {display_style}>{header}</h3><div class='content' {display_style}>{content}</div>"));
    }

    let javascript = r#"
        <script>
        document.addEventListener('DOMContentLoaded', () => {
            let currentIndex = 0;
            const headers = document.querySelectorAll('.container > h3');
            const contents = document.querySelectorAll('.content');
            const container = document.querySelector('.container');
            const buttonContainer = document.createElement('div');
            buttonContainer.classList.add('button-container');
            buttonContainer.innerHTML = '<button id="prev">Previous</button><button id="finish">Finish</button><button id="next">Next</button>';
            container.appendChild(buttonContainer);
        
            const nextButton = document.getElementById('next');
            const prevButton = document.getElementById('prev');
            const finishButton = document.getElementById('finish'); 

        
            function showContent(index) {
                headers.forEach((header, i) => {
                    header.style.display = i === index ? 'block' : 'none';
                });
                contents.forEach((content, i) => {
                    content.style.display = i === index ? 'block' : 'none';
                });
            }
        
            nextButton.addEventListener('click', () => {
                if (currentIndex < contents.length - 1) {
                    currentIndex++;
                    showContent(currentIndex);
                }
            });
        
            prevButton.addEventListener('click', () => {
                if (currentIndex > 0) {
                    currentIndex--;
                    showContent(currentIndex);
                }
            });

            finishButton.addEventListener('click', () => {
                // Hide all headers and contents
                headers.forEach(header => header.style.display = 'none');
                contents.forEach(content => content.style.display = 'none');
                // Optionally, hide the button container or just the finish button itself
                buttonContainer.style.display = 'none'; 
                // Create a new element for the final text or select an existing one
                const finalText = document.createElement('div');
                finalText.innerHTML = `
                <p>Explain what WebAssembly is and its relevance to building cloud applications in your own words:</p>
                <input type="text" id="userInput" placeholder="Your explanation here..." style="width: 100%; padding: 0.5em; margin-top: 0.5em; margin-bottom: 0.5em;">
                <button id="submitAnswer">Submit</button>`;        
                container.appendChild(finalText); 
            });
    
        
            showContent(0); 
        });
        
        </script>
        "#;

    let full_html = format!(r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <title>Learn WASM</title>
            <style>{css}</style>
        </head>
        <body>
            <h1>Learn WASM</h1>
            <div class="container">
                {content_html}
            </div>
            {javascript}
        </body>
        </html>
        "#);


    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(full_html)
        .build())
}
