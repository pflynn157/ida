
#OUTPUT
#Answer: 15
#Answer: 9
#Answer1: 15
#Answer2: 18
#END

#RET 0

use std.text_io;

func add_two(x:int, y:int) -> int
    answer : int = x + y;
begin
    printLnStrInt("Answer: ", answer);
    return answer;
end

func main -> int
    answer1 : int = 0;
    answer2 : int = 0;
begin
    answer1 = add_two(10, 5);
    answer2 = add_two(6, 3) * 2;
    
    printLnStrInt("Answer1: ", answer1);
    printLnStrInt("Answer2: ", answer2);
    
    return 0;
end

