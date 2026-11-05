#include <boost/beast/core.hpp>
#include <boost/beast/http.hpp>
#include <iostream>

namespace http = boost::beast::http;

void run_beast_dummy() {
    http::request<http::string_body> req{http::verb::get, "/", 11};
    req.set(http::field::host, "localhost");
    req.set(http::field::user_agent, "emocxx-benchmark");

    http::response<http::string_body> res{http::status::ok, 11};
    res.set(http::field::server, "Boost.Beast");
    res.body() = "Hello, world!";
    res.prepare_payload();
}

