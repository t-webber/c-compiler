pdflatex errors.tex

new-item -itemtype directory .build | out-null

move-item tex* .build/
move-item errors.* .build/
move-item ./.build/errors.tex ./ 
move-item ./.build/errors.pdf ./ 

remove-item .build -recurse -force -erroraction silentlycontinue