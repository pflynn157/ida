
#OUTPUT
#Hello!
#Hello!
#END

#RET 0

extern func puts(s:str)

func main -> int
    str s1 = "Hello!"
    str s2 = s1
    
    puts(s1)
    puts(s2)
    
    return 0
end