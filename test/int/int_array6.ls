
#OUTPUT
#22
#25
#END

#RET 0

extern func printf(s:str, ...)

func test1
    numbers : int[10] = array;
    i : int = 5;
    x : int = 0;
begin
    numbers[i+1] = 22;
    
    x = numbers[6];
    
    printLnInt(x);
end

func test2
    numbers : int[10] = array;
    i : int = 5;
    x : int = 0;
begin
    numbers[6] = 25;
    
    x = numbers[i+1];
    
    printLnInt(x);
end

func main -> int
begin
    test1();
    test2();
    
    return 0;
end
