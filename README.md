# estream

If you are a Vim user, estream can help you unlock the power of the quickfix window without dealing with the pain of Vim's `errorformat`.

[![asciicast](https://asciinema.org/a/ymMZ174YOTlHSQqmVNwb03sld.svg)](https://asciinema.org/a/ymMZ174YOTlHSQqmVNwb03sld)

### Installation

Using vim-plug:

```vim
Plug 'JoshMcguigan/estream', { 'do': 'bash install.sh v0.1.2' }

" estream doesn't directly depend on asyncrun, but they work well together
Plug 'skywind3000/asyncrun.vim'
```

### What does it do?

estream works as part of a small suite of tools and configuration to achieve the following:

 - Run a compiler, linter, or test suite asynchronously either on request or anytime a file is saved
 - View the output of the command quickly and easily (without leaving Vim)
 - When the output includes a reference to a specific location in a file, easily jump to that file location

estream formats the output from your compiler, linter, test suite, etc, so it can be easily parsed by Vim and displayed interactively in the quickfix menu. 

### Setup

First install estream as described above. Then install [asyncrun.vim](https://github.com/skywind3000/asyncrun.vim); we'll use this to run commands asynchronously (without locking the editor while the command runs). Finally, open a project in Vim and enter the following commands:

```vim
:set errorformat=%f\|%l\|%c,%f\|%l\|,%f\|\| " This describes estream's output format to Vim
:AsyncRun cargo test |& $VIM_HOME/plugged/estream/bin/estream " Replace cargo test with whichever command you use to run tests
:vert botright copen " Open the quickfix window
```

Now temporarily modify a test so that it fails, and re-trigger the `AsyncRun`. You'll see that Vim (with help from estream) has identified the relevant file locations. You can quickly jump through these locations with `:cnext` and `:cprev`.

This setup accomplishes the goals we described above, but using it is still a bit verbose. I've setup my personal [vimrc](https://github.com/JoshMcguigan/dotvim/blob/master/vimrc) with a few simple shortcuts that really improve this workflow.

### Alternatives

#### :terminal and a standard file watcher

This was my go-to setup for a long time. I'd use watchexec to trigger a test run when I changed any file, and I'd run it inside the Vim terminal. The downside to this approach is that when there is an error reported, you have to manually navigate to the appropriate file/line to fix it.

#### :make

Vim has a built in command that accomplishes most of these goals. Unfortunately is has a few drawbacks that make it not well suited for my particular workflow. The `:make` command can be used to run an external program or script and put the output of that program into the quickfix window. It can even handle parsing that output to display and jump to errors.

The downside to `:make` is that it is synchronous, so you cannot keep working while the command is running. It also relies on Vim's `errorformat` to do the parsing, which can be nice, because it is often set by language plugins to match the output of a particular suite of tools. In practice, there are some downsides which we'll get further into in the next section.

#### vim-dispatch or asyncrun.vim

I tried two plugins to work around the synchronous nature of `:make`, both of which provide basically an asynchronous version of the same command. I particularly like `asyncrun.vim`, and in fact still use it alongside estream. The downside of these tools is they still rely on Vim's `errorformat` to parse file names and line numbers out of the output, and this has a couple downsides.

If your compiler/test suite output errors or warnings which span multiple lines, `errorformat` cannot parse this in real time. This means if you want correctly formatted output you have to re-trigger the parsing after the command is finished running. This is problematic for me, because I like to leave the quickfix window open almost always, so I can get immediate feedback.

In a way I am using the quickfix window as an enhanced terminal window, and for that reason I want to see the full output of my commands. This doesn't quite align with the `errorformat` implementation, which can eliminate or restructure lines of output.

Finally, `errorformat` is configured within Vim with scanf-like patterns, which are difficult to test/debug, and lack power and flexibility.

### So what again does estream do?

Rather than relying on a complex `errorformat` configuration within Vim to associate compiler or test suite output to file names and line numbers, estream does the parsing and outputs the file name and line number information into a format that can be parsed by a very short/simple Vim `errorformat`. Basically, all of the complex pattern matching logic that would have gone into your `errorformat` instead lives in estream, which is much easier to develop and test. estream can also do things that `errorformat` cannot do, like check if a file exists before outputting a suggestion for you to open the file (this is something that happened all the time when I was using a traditional `errorformat` based setup). Because of this extra check, the pattern matching logic within estream can be simplified significatly.

### Example vimrc

```vim
" Ensure you have asyncrun.vim and estream installed before applying this configuration

" Set global error format to match estream output
set errorformat=%f\|%l\|%c,%f\|%l\|,%f\|\|
" Use global error format with asyncrun
let g:asyncrun_local = 0

" Pipe any async command through estream to format it as expected
" by the errorformat setting above
" example: `:Async cargo test`
command -nargs=1 Async execute "AsyncRun <args> |& $VIM_HOME/plugged/estream/bin/estream"
nnoremap <leader>a :Async 
nnoremap <leader>s :AsyncStop<CR>

" Create a file watcher, primarily used with Async using the mapping below
command -nargs=1 Watch augroup watch | exe "autocmd! BufWritePost * <args>" | augroup END
command NoWatch autocmd! watch

" Use to run a command on every file save, pipe it through estream
" and view it in the quickfix window.
" example: `:Watch Async cargo test`
nnoremap <leader>w :Watch Async 
nnoremap <leader>nw :NoWatch<CR>
```

With this setup, `:Async cargo test` can be used to trigger a one-off action, and `:Watch Async cargo test` can be used to setup an action to run after every file save. The mappings `<leader>a` and `<leader>w` make it fast to use these commands. Note that in either case, the output will be piped through estream before going to the quickfix window.

I also configure a few mappings to quickly open/close/toggle the quickfix window and the Vim terminal. You can check those out [here](https://github.com/JoshMcguigan/dotvim/blob/master/vimrc).

### Developer notes

To publish a new release, update the version in `Cargo.toml`, then create a new release on GitHub. CI will build and publish new binaries. Then update the install instructions on the readme.

To test a local build of the software, run `cargo build`, then manually copy the binary to `$VIM_HOME/plugged/estream/bin/estream`.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
