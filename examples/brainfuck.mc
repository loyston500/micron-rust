s:-2 i
s:-4 ""
s:-3 0

;filter
    s:-5 x:.-2 .-3

    ?:=:.-5 "+" s:-4 a:.-4 .-5
    ?:=:.-5 "-" s:-4 a:.-4 .-5
    ?:=:.-5 ">" s:-4 a:.-4 .-5
    ?:=:.-5 "<" s:-4 a:.-4 .-5
    ?:=:.-5 "." s:-4 a:.-4 .-5
    ?:=:.-5 "," s:-4 a:.-4 .-5
    ?:=:.-5 "[" s:-4 a:.-4 .-5
    ?:=:.-5 "]" s:-4 a:.-4 .-5

    s:-3 a:.-3 1
    ?:.-5 j:"filter"

[now slot -4 holds the filtered input]

[cleaning up used slots]
s:-5 .-6969
s:-3 .-6969 
s:-2 .-6969

[p:.-4]

[fill slots from 0 to 10K with 0s]

s:-2 10000
s:-3 0

;fill
    s:.-3 0
    s:-3 a:.-3 1
    ?:=:.-3 .-2 j:"next"
    j:"fill"

[clean up]
s:-2 .-6969
s:-3 .-6969

[time to run ig]
;next

s:-2 0 [pointer]
s:-5 0

;run 
    s:-3 x:.-4 .-5 [extract char from the string one by one]
    ?:=:.-3 "" j:"exit"
    ?:=:.-3 "+" s:.-2 a:g:g:-2 1
    ?:=:.-3 "-" s:.-2 a:g:g:-2 -1

    ?:=:.-3 "." w:c:g:g:-2
    ?:=:.-3 "," s:.-2 c:i

    ?:=:.-3 ">" s:-2 a:.-2 1
    ?:=:.-3 "<" s:-2 a:.-2 -1

    ?:=:.-3 "[" f:"open"
    ?:=:.-3 "]" f:"close"
    s:-5 a:.-5 1
    j:"run"

;open
    s:-6 0
    ?:=:g:g:-2 0 j:"loop1"
    r:0

    ;loop1
        ?:=:.-3 "[" s:-6 a:.-6 1
        ?:=:.-3 "]" s:-6 a:.-6 -1
        
        ?:=:.-6 0 r:0
        s:-5 a:.-5 1
        s:-3 x:.-4 .-5
        j:"loop1"



;close
    s:-6 0
    ?:=:g:g:-2 0 r:0
    
    ;loop2
        ?:=:.-3 "]" s:-6 a:.-6 1
        ?:=:.-3 "[" s:-6 a:.-6 -1

        ?:=:.-6 0 r:0
        s:-5 a:.-5 -1
        s:-3 x:.-4 .-5
        j:"loop2"

;exit
$
