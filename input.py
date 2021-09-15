import emb
import libmeliplugin

print(dir(libmeliplugin))

print("Number of arguments", emb.s("Im a python string"))


k = libmeliplugin.CharKeyInput("c")

emb.o(k)
