import data from "./targets.json"

const server = Bun.serve({
    routes: {
        "/api": _ => {
            console.log("API request received")
            return new Response(JSON.stringify(data), {
                headers: {
                    "Content-Type": "application/json",
                },
            })
        }
    },
    port: 4000,
})

console.log(`API server running at http://localhost:${server.port}/api`)