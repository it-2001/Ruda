import "#io"
import "#fs"
import "#string"

import "danda.rd"

/*
fun a<T>(arg: T): T {
    return arg
}*/


fun main() {
    let file = fs.File("test.txt")
    io.println(file.read())
}