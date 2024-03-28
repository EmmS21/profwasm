use serde_json::Value;
use spin_sdk::{http::{IntoResponse, Method, Request, Response}, http_component, llm::{infer_with_options, InferencingModel::Llama2Chat},};
use serde::{Deserialize, Serialize};

struct UserInput {
    userInput: String,
}

#[http_component]
async fn handle_profwasm(req: Request) -> anyhow::Result<impl IntoResponse> {
    let css = include_str!("../static/style.css");

    match req.method() {Method::Get=>{
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
                    headers.forEach(header => header.style.display = 'none');
                    contents.forEach(content => content.style.display = 'none');
                    buttonContainer.style.display = 'none'; 
                    const finalText = document.createElement('div');
                    finalText.innerHTML = `
                    <p>Explain what WebAssembly is and its relevance to building cloud applications in your own words:</p>
                    <input type="text" id="userInput" placeholder="Your explanation here..." style="width: 100%; padding: 0.5em; margin-top: 0.5em; margin-bottom: 0.5em;">
                    <button id="submitAnswer">Submit</button>`;        
                    container.appendChild(finalText); 
                
                const submitButton = document.getElementById('submitAnswer');
                const userInputField = document.getElementById('userInput');
                submitButton.addEventListener('click', () => {
                    const userInput = userInputField.value;
                    submitButton.style.display = 'none';
                    const pleaseWaitMessage = document.createElement('p');
                    pleaseWaitMessage.id = 'pleaseWaitMessage';
                    pleaseWaitMessage.textContent = 'Please Wait...';
                    userInputField.after(pleaseWaitMessage);
                
                    fetch('/handle_profwasm', {
                        method: 'POST',
                        headers: {
                            'Content-Type': 'application/json'
                        },
                        body: JSON.stringify({ userInput: userInput })
                    })
                    .then(response => {
                        if (!response.ok) {
                            throw new Error('Network response was not ok');
                        }
                        return response.text();
                    })
                    .then(data => {
                        pleaseWaitMessage.style.display = 'none';
                        const percentageMatch = data.match(/Percentage: (\d+)/);
                        const percentage = percentageMatch ? percentageMatch[1] : "N/A";
                
                        userInputField.style.display = 'none';
                        submitButton.style.display = 'none';
                        finalText.innerHTML = `<p>Let me score how well you understand this topic:</p>`;
                
                        const responseContainer = document.createElement('div');
                        responseContainer.style.cssText = `
                            padding: 1em;
                            margin-top: 20px;
                            background-color: #f0f0f0;
                            border-radius: 8px;
                            box-shadow: 0 2px 4px rgba(0,0,0,0.1);`;
                        responseContainer.textContent = data; 
                                    
                        const container = document.querySelector('.container');
                        container.appendChild(responseContainer);
                    })
                    .catch(error => console.error('Error during POST request:', error));                });
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
    },
    Method::Post => {
        let body = req.into_body();
        let user_input: Value = serde_json::from_slice(&body)?; 
       
        let text = user_input.get("userInput").and_then(|v| v.as_str()).unwrap_or("");
        const PROMPT: &str = r#"\
        <<SYS>>
        You are a bot, you will only give me a final percentage score out of 100 for my ability to clearly explain what WebAssembly is and it's relevance to cloud compute. Keep you explanation concise, no more than 4 sentences and return a score out of 100 for my understanding.
        <</SYS>>
        <INST>
        Simply return how I could improve my explanation in no more than 3 lines a final score out of 100
        User: Here is the explanation in my own words: {SENTENCE}
        </INST>
        "#;

        let prompt = PROMPT.replace("{SENTENCE}", text);

        let infer_result = match infer_with_options(
            Llama2Chat,
            &prompt,
            spin_sdk::llm::InferencingParams {
                max_tokens: 256,
                ..Default::default()
            },
        ) {
            Ok(infer_result) => infer_result,
            Err(err) => {
                return Ok(Response::builder()
                    .status(500)
                    .body("Internal Server Error")
                    .build());
            }
        };

        println!("Llama2 response: {:?}", infer_result);

        if infer_result.text.is_empty() {
            return Err(anyhow::anyhow!("LLAMA response is empty"));
        }

        let response_body = infer_result.text;

        return Ok(Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(response_body)
        .build());
    }
    Method::Head => todo!(),
    Method::Post => todo!(),
    Method::Put => todo!(),
    Method::Delete => todo!(),
    Method::Connect => todo!(),
    Method::Options => todo!(),
    Method::Trace => todo!(),
    Method::Patch => todo!(),
    Method::Other(_) => todo!(), }

    }
