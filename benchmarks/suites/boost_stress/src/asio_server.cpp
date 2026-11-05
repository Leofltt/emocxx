#include <boost/asio.hpp>
#include <iostream>

void run_asio_dummy() {
    boost::asio::io_context ioc;
    boost::asio::steady_timer timer(ioc, boost::asio::chrono::milliseconds(1));
    timer.async_wait([](const boost::system::error_code& ec) {
        if (!ec) {
            std::cout << "Boost Asio timer fired\n";
        }
    });
    ioc.poll();
}

