
#OUTPUT
#X: FFFFA1B1
#END

#RET 0

extern func printf(s:str, ...)

func main -> int
    short[10] numbers = array
    
    numbers[3] = 0xA1B1
    
    short x = numbers[3]
    
    printf("X: %X\n", x)
    
    return 0
end