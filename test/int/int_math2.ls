
#OUTPUT
#X1: 22
#X2: 132
#END

#RET 0

extern func printf(s:str, ...)

func main -> int
    int x = 22
    
    printf("X1: %d\n", x)
    
    int y = 3
    x = 44 * y
    
    printf("X2: %d\n", x)
    
    return 0
end