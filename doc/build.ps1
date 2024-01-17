rmdir -recurse .build

pdflatex errors.tex

mkdir -p .build | Out-Null

mv tex* .build/
mv errors.* .build/

mv ./.build/errors.tex ./ 
