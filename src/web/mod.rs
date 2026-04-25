use actix_web::{HttpResponse, Responder, Scope, web};

pub fn routes() -> Scope {
    web::scope("")
        .service(web::resource("/").route(web::get().to(index)))
        .service(web::resource("/send").route(web::get().to(send_page)))
        .service(web::resource("/status").route(web::get().to(status_page)))
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>ReChat Sender</title>
            <style>
                body { font-family: Arial, sans-serif; margin: 20px; }
                h1 { color: #333; }
                .nav { margin: 20px 0; }
                .nav a { margin-right: 10px; text-decoration: none; color: #0066cc; }
                .nav a:hover { text-decoration: underline; }
            </style>
        </head>
        <body>
            <h1>ReChat Sender</h1>
            <div class="nav">
                <a href="/">Home</a>
                <a href="/send">Send Message</a>
                <a href="/status">Status</a>
            </div>
            <p>Welcome to ReChat Sender! This is a web interface for sending messages.</p>
        </body>
        </html>
    "#,
    )
}

async fn send_page() -> impl Responder {
    HttpResponse::Ok().body(r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Send Message - ReChat Sender</title>
            <style>
                body { font-family: Arial, sans-serif; margin: 20px; }
                h1 { color: #333; }
                .nav { margin: 20px 0; }
                .nav a { margin-right: 10px; text-decoration: none; color: #0066cc; }
                .nav a:hover { text-decoration: underline; }
                form { margin: 20px 0; }
                label { display: block; margin: 10px 0; }
                input, select, textarea { width: 100%; padding: 8px; }
                button { margin: 10px 0; padding: 10px 20px; background-color: #0066cc; color: white; border: none; border-radius: 4px; }
                button:hover { background-color: #0052a3; }
            </style>
        </head>
        <body>
            <h1>Send Message</h1>
            <div class="nav">
                <a href="/">Home</a>
                <a href="/send">Send Message</a>
                <a href="/status">Status</a>
            </div>
            <form id="messageForm">
                <label for="messageType">Message Type:</label>
                <select id="messageType" name="messageType">
                    <option value="Text">Text</option>
                    <option value="Image">Image</option>
                    <option value="File">File</option>
                </select>
                <label for="recipient">Recipient:</label>
                <input type="text" id="recipient" name="recipient" placeholder="Enter recipient">
                <label for="content">Content:</label>
                <textarea id="content" name="content" rows="4" placeholder="Enter message content"></textarea>
                <button type="submit">Send Message</button>
            </form>
            <div id="result"></div>
            <script>
                document.getElementById('messageForm').addEventListener('submit', async (e) => {
                    e.preventDefault();
                    const formData = new FormData(e.target);
                    const data = {
                        message_type: formData.get('messageType'),
                        content: formData.get('content'),
                        recipient: formData.get('recipient')
                    };
                    const response = await fetch('/api/messages', {
                        method: 'POST',
                        headers: {
                            'Content-Type': 'application/json'
                        },
                        body: JSON.stringify(data)
                    });
                    const result = await response.json();
                    document.getElementById('result').innerHTML = JSON.stringify(result, null, 2);
                });
            </script>
        </body>
        </html>
    "#)
}

async fn status_page() -> impl Responder {
    HttpResponse::Ok().body(r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Status - ReChat Sender</title>
            <style>
                body { font-family: Arial, sans-serif; margin: 20px; }
                h1 { color: #333; }
                .nav { margin: 20px 0; }
                .nav a { margin-right: 10px; text-decoration: none; color: #0066cc; }
                .nav a:hover { text-decoration: underline; }
                #messageStatus { margin: 20px 0; }
                input { padding: 8px; margin-right: 10px; }
                button { padding: 8px 16px; background-color: #0066cc; color: white; border: none; border-radius: 4px; }
                button:hover { background-color: #0052a3; }
                #result { margin: 20px 0; padding: 10px; border: 1px solid #ddd; border-radius: 4px; }
            </style>
        </head>
        <body>
            <h1>Message Status</h1>
            <div class="nav">
                <a href="/">Home</a>
                <a href="/send">Send Message</a>
                <a href="/status">Status</a>
            </div>
            <div id="messageStatus">
                <input type="text" id="messageId" placeholder="Enter message ID">
                <button onclick="checkStatus()">Check Status</button>
            </div>
            <div id="result"></div>
            <script>
                async function checkStatus() {
                    const messageId = document.getElementById('messageId').value;
                    if (!messageId) {
                        alert('Please enter a message ID');
                        return;
                    }
                    const response = await fetch(`/api/messages/${messageId}`);
                    const result = await response.json();
                    document.getElementById('result').innerHTML = JSON.stringify(result, null, 2);
                }
            </script>
        </body>
        </html>
    "#)
}
