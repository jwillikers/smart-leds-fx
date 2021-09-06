target extended-remote /dev/ttyBmpGdb
monitor swdp_scan
attach 1

set print asm-demangle on

break DefaultHandler
break rust_begin_unwind

load
