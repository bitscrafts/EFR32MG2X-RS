
book creation with detailded implementation pac, hal, bsp and protocol stacks (using rust).
- you should use the technical-book-writer skill (project local) in every iteration.
- this should be a going on writing experience! the book will be increased/optimized with time.
- essentilly it will be a technical book/manual explaining in detail a new hardwere suport in rust.
- create in the workspace root the folder book/{latex,project}.
- analyse the whole project (also check the docs folder) for insights.
- you are only allowed to write to the book folder.
- i got the idea, in adition to other examples, to provide a singular example on each phase, beginning with bare metal, pac, hal and bsp implemetation displaying and analysing the degree of complexity of each phase, could be a simple implementation like a common blink, or a led and a button to control it, showing the complexity and dificulty of each phase!
- it should be based in our real EFR32MG24 implementation.
- it should be created incrementally, implement the pac and then create an examples folder with level related examples (building and testing them).
- in near future we will document all the work we did supporting a new hardware 'EFR32MG24' in rust, it should be a detailed instruction book.
- in the book folder we will have TOC.yml and README.md. In the latex folder we will have the complete chapter/subchapter latex documentation, and in the project folder we will copy relevant pieces of code to re-implement the project looking forward the book writing. Suggest a better approach.
- we will discuss on the approach of writing the book, initially ultra thinking about the creation of the TOC.yml 'table of contents', possible paths, suggestions and best approach. And the README.md with detailed instructions on how to do the book writing.
- the TOC.yml should have a detailed structure of the book chapters/subchapters, the approach, the content and should be an llm. suggest the json structure to fill all the needed information.
- the book should be detailed enought to allow anyone with minimal competence to try to support a new hardware/mcu using rust.
- it should detail all implementation options, file systems structure, order of implementation, options taken when implementing among others.
- it should have detailed instructions used on the efr32mg24 comparing and analysing the reader implementaion.
- create a retailed README.md with plan and related info and put the book folder in gitignore!
- we use the project folder to copy (from out working implementation) code to put in the book.
- we could implement the code using tdd 'test driven development' to make it more didactical approach on the book.
- we will intensively use code quotes in the book showing text edits and commands.
- we will intensively use diagrams, graphs in the book (use tikz).
- the book should also be able to answer questions like "why implementeded and its location" for each feature.
- the pdf compilation will be made remotely (in a container) where the llm must use rsync to push the latex folder to be build remotely.

we will return to the book implementation in near future.