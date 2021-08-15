s:0 10 [set value of slot 0]
s:1 10 [set value of slot 1]

w:"Value of slot 0 is: "
p:.0

w:"Value of slot 1 is: "
p:.1

s:2 =:.0 .1
?:.2 j:"true"

p:"Both are not equal"
$

;true
p:"Both are equal"


