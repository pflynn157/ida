
#OUTPUT
#Syntax Error: Invalid constant or variable name: answerrr
# -> [17] printf("Answer: %d\n", answerrr)
#
#END

#RET 0

extern func puts(s:str)

func main -> int
    int x = 6
    int y = 3
    
    int answer = x * y
    printf("Answer: %d\n", answerrr)
    
    return 0
end