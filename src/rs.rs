//! ## Reed-Solomon error-correction codes (BCH-view)
//!
//! [Reed-Solomon error-correction][rs-wiki] is a scheme for creating
//! error-correction codes (ECC) capable of detecting and correcting multiple
//! byte-level errors.  By adding `n` extra bytes to a message, Reed-Solomon is
//! able to correct up to `n` byte-errors in known locations, called "erasures",
//! and `n/2` byte-errors in unknown locations, called "errors".
//!
//! Reed-Solomon accomplishes this by viewing the entire codeword (message + ecc)
//! as a polynomial in `GF(256)`, and limiting valid codewords to polynomials that
//! are a multiple of a special "generator polynomial" `G(x)`.
//!
//! ``` rust
//! use gf256::rs::rs255w223;
//! 
//! // encode
//! let mut buf = b"Hello World!".to_vec();
//! buf.resize(buf.len()+32, 0u8);
//! rs255w223::encode(&mut buf);
//! 
//! // corrupt
//! buf[0..16].fill(b'x');
//! 
//! // correct
//! rs255w223::correct_errors(&mut buf)?;
//! assert_eq!(&buf[0..12], b"Hello World!");
//! # Ok::<(), rs255w223::Error>(())
//! ```
//!
//! Note this module requires feature `rs`.
//!
//! A fully featured implementation of Reed-Solomon error-correction can be found in
//! [`examples/rs.rs`][rs-example]:
//!
//! ``` bash
//! $ RUSTFLAGS="-Ctarget-cpu=native" cargo run --features thread-rng,lfsr,crc,shamir,raid,rs --example rs
//!
//! testing rs("Hello World!")
//! dimension = (255,223), 16 errors, 32 erasures
//! generator = 01744034ae367e10c2a221219db0c5e10c3b37fde4942fb3b9188afd148e37ac58
//! rs_encode           => Hello World!.......n_...K...4....%...a.....5  48656c6c6f20576f726c642185a6adf8bd15946e5fb607124bbd11d33414a706d625fd84c26181a78a15c935
//! corrupted (32,0)    => xxlxoxxorxxxxxx..xxxx.xxKxxx4xxxxxx.xxxx.xx5  78786c786f78786f72787878787878f8bd78787878b678784b7878783478787878787884787878788a787835
//! rs_correct_erasures => Hello World!.......n_...K...4....%...a.....5  48656c6c6f20576f726c642185a6adf8bd15946e5fb607124bbd11d33414a706d625fd84c26181a78a15c935
//! corrupted (0,16)    => Hellx xoxld!..xxx..n_..xK.xx4xxxx%x..a.x.x.5  48656c6c7820786f786c642185a678787815946e5fb607784bbd78783478787878257884c26181788a78c935
//! rs_correct_errors   => Hello World!.......n_...K...4....%...a.....5  48656c6c6f20576f726c642185a6adf8bd15946e5fb607124bbd11d33414a706d625fd84c26181a78a15c935
//! corrupted (28,2)    => xxxlxxxxxxd!x.x..xxx_xxxKx.x4.x.xx.xxxxxx.xx  7878786c787878787878642178a678f8bd7878785f7878784b781178341478067878fd787878787878157878
//! rs_correct          => Hello World!.......n_...K...4....%...a.....5  48656c6c6f20576f726c642185a6adf8bd15946e5fb607124bbd11d33414a706d625fd84c26181a78a15c935
//! 
//! bit corrupted image (errors = 48/3072, 1.56%):
//! 
//!                       ..:::::::::::...        '        '.:::....:::.:.:.'':.'...'::: :   :' ''  '''
//!                    .::::::::'::::::::::..         '     ::::::::::::  .:....' . '.:...  :' :'':': .'
//!                  .':::::::::::::::::::::::.             ::::::::::' . :..:..:' : '.':::  :. :'' '  :
//!                .:::::::::::::::::::::::::::.         . ::::::::::   :: '::' .: '' ''. : :: .:..::. '
//!               .:::::::::::::::::::::::::::::.    ... :: ''::::::'   ::':. ':' .. '.''::'::: ':''...:
//!               ::::::::::::::::::::'::::::::'' .. '::: '     '''     ''..: :'.''::.   .:.' .'': '. .:
//!              ::::: ::::::::' ::::::'::::''..::::: '                  : .: .'. :  :::.'.:':.:':: .. :
//!              :::::::::::::::::::::::'' ..:::::'''                   ' :... .': ::.''':''.''. . '  ..
//!           .  ::::::::::::::::':.'''..:::.:'' .                       : :..':::.:. : .:' :.   .':'.':
//!          ..: ::::::::::::::''' ..:::::'' ..:::                      :' . :.' .'.'::  ' ::' ':  .:.
//!       ..:::'  :::::::::'' ..::::::'' ..:::::::                      :''.  '.'::'.:  ' .:'' .:.'.::''
//!     .:::'     ':::'' ...::::::''....::::::::'                        '' ' .'::...' :':...':.. . .' :
//!     ::'     '   ...::::::'' ..:::::::::::::'                        . .. ' .:::.'':::.  .':''':::...
//!          ....:::::::'' ..::::::::::::::.::'   .                     : '.   .': :. .:.': . .'  .  ::'
//!     '::::::::'''    :::::::::::::::::::''                            '.. .:'::: ': ::'::. . '.': : '
//!            .          '':::::::::::'''               .              .:.':..  ''.' : : ':'''.': '.:':
//! 
//! byte corrupted image (errors = 49/384, 12.76%):
//! 
//!                       ..    ::::    ..                  .:::....:::.:.:.'':.'.:.':::::::::::''  '''
//!                    .:::::::::::::::::::..       ::::::::::::    ::::  .:::::'.. '.:...  :' :    : .'
//!                  .::::::::::::::::::::::::.         ::::::::::::::' . :..:..:' ::::::::  :.':'' ::::
//!                .:::::::::::::::::::::::::::.         . ::::::::::   :: '::' .: '' ''. : :: .    :. '
//!     ::::      .:::::    :::::::::::::::::::::::: ... :: :'::::::'   ::':. ':' .:::::'::'::: ': '...:
//!               ::::::::::::::    :::::::::::'' ..    : '     '''     :::::.:'    :.   .:.    ': '. .:
//!              :::::::::::::::::::::::::::''..::::: '             :::: : .: .'. :  :::    ':.:':: .. :
//!              :::::::    ::::::::::::'' ..:::::''::::                ' :... .    :.''':' .''. . '  ..
//!         :::: :::::::::::::::::::'' ..::::::: .              :::::::: : :..':::.:. : .:' :.   .':'.':
//!          ..: ::::::::::::::''' ..:::::'' ..:::                      :' .:::: .'.'::. ' ::' ':  .:.
//!       ..:::'  ::::::    ' ..    ::'' ..:::::::                      :' .  '.'::'.:  : .::::::.'.:::'
//!      :::'     ':::'' ...::::::''...:    :::::                        '' ' .'    .' :':...':.. . .' :
//!     ::'         ...::::::'' ..::::::::::                            . ..:::::::.''::::::.':''':::...
//!          ....:::::::'' ..:::::::::::::::::'                         : '.'  .': :. .:.': . .'  .
//!     '::::::::'''    :::::::::::::::::::''                   :::::::: '.. .:     ': ::'::. ' '.': : '
//!                     ::::::::::::    ''                              .:.':.. '''.  : : ':'':.': '.:':
//! 
//! corrected:
//! 
//!                       ..:::::::::::...                  .:::....:::.:.:.'':.'.:.'::: :   :' ''  '''
//!                    .:::::::::::::::::::..               ::::::::::::  .:....'.. '.:...  :' :'':': .'
//!                  .::::::::::::::::::::::::.             ::::::::::' . :..:..:' : '.':::  :.':'' '  :
//!                .:::::::::::::::::::::::::::.         . ::::::::::   :: '::' .: '' ''. : :: .:..::. '
//!               .::::::::::::::::::::::::::::::    ... :: :'::::::'   ::':. ':' .: '.:'::'::: ': '...:
//!               :::::::::::::::::::::::::::::'' .. '::: '     '''     ''..:.:'.''::.   .:.' .'': '. .:
//!              :::::::::::::::::::::::::::''..::::: '                  : .: .'. :  :::.'.:':.:':: .. :
//!              :::::::::::::::::::::::'' ..:::::''                    ' :... .': ::.''':' .''. . '  ..
//!              :::::::::::::::::::'' ..:::::'' .                       : :..':::.:. : .:' :.   .':'.':
//!          ..: ::::::::::::::''' ..:::::'' ..:::                      :' . :.' .'.'::. ' ::' ':  .:.
//!       ..:::'  :::::::::'' ..::::::'' ..:::::::                      :' .  '.'::'.:  : .:'' .:.'.:::'
//!      :::'     ':::'' ...::::::''...::::::::::                        '' ' .'::...' :':...':.. . .' :
//!     ::'         ...::::::'' ..:::::::::::::'                        . .. ' .:::.'':::.  .':''':::...
//!          ....:::::::'' ..:::::::::::::::::'                         : '.'  .': :. .:.': . .'  .  ::'
//!     '::::::::'''    :::::::::::::::::::''                            '.. .: ::: ': ::'::. ' '.': : '
//!                       '':::::::::::'''                              .:.':.. '''.  : : ':'':.': '.:':
//!
//! bit corrupted image (errors = 106/3456, 3.07%):
//! 
//!               '        .:::::::::::...        .         .:::....:':.:.:.'':.'.:.'::' :   :' ''  '''
//!       '        .    ::::::::::::::::::'..               ::::::::::':  .:....'.. '.:...  :' :'':': .'
//!                  .:::::::::::::::::::::::::   .     .   ::':::::::' . '..:...' : '.':::  :.':'' '  :
//!                .::::.::::::::::::.:::.:::::.   .     . ::::::::::   :: '::' .: '' ''. . :: .'..::. '
//!               .::::':::::::::::::::::::::::::    ... :: :'::::::'   ::':  ':' .:  .:'::'::: ': '...:
//!               :::::::::::::::::::::::::::::'' .: '::' '     '''    '''. :.:'.' ::  . .:.' .'': '...:
//!      '       ::'::::::::::::::::::::::::''..::::: '    '             : .:'.'. : .:::.'.:':.:':: .. :
//!     '        ::::::::::::::::::::::::' ..:::::''             '      ' :... .'' ::.''':' .':: . '  ..
//!              ::::::::::::::.:::: ' ...:::''' .                       : ...':::.:. : .:'.:.   .':'.':
//!          ..: ::::::::.'::::''' ..:::::'''..:::        .             :' . :.' .'.'::. ' ::' ':  .:.
//!       ..:.''  :::::::::'' ..::::::'' :.::::.::                      :' .  '.'::'.:  : .:''..:.'.:::'
//!      .::'.    ':::'' ....:::::''...::::::::::     '        '     .  ''' ' .'::...' :':...':..  ..' :
//!     ::'     '   ...::.::.'' ..'::::::::::.:            '            .  : ' .:::.'':::.'..':''':::...
//!          ....:::::::'' ..:::::::::':.:::::                          : '.'  .': :. ::.': :'.'  .  ::'
//!     ':::'::::'''    :::::::::::.:::::::''        '  '             '  '.. .: ':: ': ::':..   '.'' : '
//!                       '':::::::::::'''                              .:.':.: '''.  . : ':'':.':.'.: '
//!     :    .  . ' '  . .   . '     ' '.'  :  . '    :       ''      ':: : .'.:''.:.: ::.. ': '... ::':
//!     '   '.  '. .   ''.  ''   '  .:    .:' ::  '   '   '. '    '     .  : ...'':'  ::.: .:.':''.'''
//! 
//! byte corrupted image (errors = 81/384, 21.09%):
//! 
//!                       ..::::        ..                  .:::....:::.:.:.'':.         :   :' ''  '''
//!             ::::   .:::::::::::::::::::.::::    ::::        ::::::::  .:....'.. '.:.:::::' :::::: .'
//!                     ::::::::::::        ::.                 ::::::' . :..:..:' ::::::::  :.':'' '  :
//!     ::::       .::::::::::::::::::::    :::.         . ::::::::::   :: '::' :::::::::::::: .:..::. '
//!               .:::::::::    ::::    :::::::::    ... :: :'::::::'   ::':. '::::: '.:'::'::: ': '...:
//!         ::::  :::::::::::::::::::::::::::::'' .. '::::::::::::::::::''..    .''::.   .:.' .'': '::::
//!              :::::::::::::::::::::::::::''..::::::::                 : .: .'. :  :::.'.:':.:':: .. :
//!              :::    ::::::::::::    '' ..:::::''            ::::    ' :... .': ::.''':' .''. . '  ..
//!              :::::::::::::::::::'' ..:::::'' .              :::::::: : :    ::.:. : :::::.   .':'.':
//!          ..:    :::::::::::''' ..:::::'' ..:::  ::::                :' . :.' .'.'::. ' ::' ':  .:.
//!       ..    :::::::::::'::::::::::'' ..:::::::          ::::        :' .  '.'::'::::: .:'' .:.'.:::'
//!      :::'     ':::'' ...::::::''...::::::::::       ::::::::         '' ::::::::.' :':...':.. . ::::
//!     ::'         ...::::::''     :::::::::::'                        . .. ' .:::.''::    .':''':::...
//!         ::::::::    '' ..:::::::::::::::::'         ::::            : '.'  .': :. .:.': . .':::: ::'
//!     '::::::::'''        :::::::::::::::''                   ::::         .: ::: ': :    ::::     : '
//!                 ::::  '':::::::::::'''  ::::                        .:.':..       : : ':'':.': '.:':
//!     :       ..' '   '.   ..      '  .'  :  . '    :      .''.'   '':' : .  :'':..: . :. :: ' .. ::':
//!        .'.' '. :.  ''.  ''   '  .:    .': ::  '  ''''.'. .'   :   ' .  : ...':::  ::.: ::.':'' '':'
//! 
//! corrected:
//! 
//!                       ..:::::::::::...                  .:::....:::.:.:.'':.'.:.'::: :   :' ''  '''
//!                    .:::::::::::::::::::..               ::::::::::::  .:....'.. '.:...  :' :'':': .'
//!                  .::::::::::::::::::::::::.             ::::::::::' . :..:..:' : '.':::  :.':'' '  :
//!                .:::::::::::::::::::::::::::.         . ::::::::::   :: '::' .: '' ''. : :: .:..::. '
//!               .::::::::::::::::::::::::::::::    ... :: :'::::::'   ::':. ':' .: '.:'::'::: ': '...:
//!               :::::::::::::::::::::::::::::'' .. '::: '     '''     ''..:.:'.''::.   .:.' .'': '. .:
//!              :::::::::::::::::::::::::::''..::::: '                  : .: .'. :  :::.'.:':.:':: .. :
//!              :::::::::::::::::::::::'' ..:::::''                    ' :... .': ::.''':' .''. . '  ..
//!              :::::::::::::::::::'' ..:::::'' .                       : :..':::.:. : .:' :.   .':'.':
//!          ..: ::::::::::::::''' ..:::::'' ..:::                      :' . :.' .'.'::. ' ::' ':  .:.
//!       ..:::'  :::::::::'' ..::::::'' ..:::::::                      :' .  '.'::'.:  : .:'' .:.'.:::'
//!      :::'     ':::'' ...::::::''...::::::::::                        '' ' .'::...' :':...':.. . .' :
//!     ::'         ...::::::'' ..:::::::::::::'                        . .. ' .:::.'':::.  .':''':::...
//!          ....:::::::'' ..:::::::::::::::::'                         : '.'  .': :. .:.': . .'  .  ::'
//!     '::::::::'''    :::::::::::::::::::''                            '.. .: ::: ': ::'::. ' '.': : '
//!                       '':::::::::::'''                              .:.':.. '''.  : : ':'':.': '.:':
//!     :    .  . ' '  . .   .       ' '.'  :  . '    :       ''      ':: : .' :''.:.: ::.. ': '... ::':
//!         '.  '. .   ''.  ''   '  .:    .'' ::  '   '   '.      '     .  : ...'':'  ::.: .:.':'' '''
//! ```
//!
//! ## How does error-correction work?
//!
//! I think it's first worth understanding how error-correction codes work in general.
//!
//! Consider the message "hello!":
//!
//! ``` text
//! hello!  68 65 6c 6c 6f 21
//! ```
//!
//! Representing, storing, transfering, etc, the message as bytes, is rather flimsy.
//! All it takes is one faulty transistor, a scratch on a disk, or a rogue neutrino
//! to flip a bit and completely change the meaning of the message.
//!
//! ``` text
//! hgllo!  68 61 6c 6c 6f 21
//! ```
//!
//! Ok, maybe _completely_ change was an exaggeration. We can still tell that the
//! original message was probably "hello!".
//!
//! But why is that?
//!
//! One explanation is that there really aren't that many 5-letter words in the
//! english language. "hgllo" isn't a word, and sure "igloo" is pretty close,
//! requiring 2 character changes, but "hello" is closer, requiring only one
//! character change.
//!
//! According to the [World English Language Scrabble Players Association][wespa],
//! there are 279,496 words in the english language. In theory, all of these
//! words should fit comfortably in 4 letters (log base 26 of 279496). But for
//! whatever reason, english has words with 5 letters, 6 letters, and sometimes
//! even more!
//!
//! The reason we use more than 4-letter words, isn't because we want to give
//! compression algorithms something to do, it's because it's much easier to
//! communicate if we allow some errors in our words. We add extra letters to
//! our words so that if a loud wind, thunderclap, or swarm of angry bees
//! introduces an error, we can still recover the original message.
//!
//! This is basically how error-correction codes work. We create a codeword by
//! adding some extra bits, the error-correction code, to our original message
//! so that there are much fewer _valid_ codewords than there are _possible_
//! codewords.
//!
//! ---
//!
//! For another silly example, consider some data checksummed using a
//! 32-bit [CRC](../crc):
//! 
//! ``` rust
//! use ::gf256::crc::crc32c;
//!
//! assert_eq!(crc32c(b"hello!", 0), 0x8c09fd5b);
//! ```
//!
//! ``` text
//! hello!...[  68 65 6c 6c 6f 21 5b fd 09 8c
//!             \-------+-------/ \----+----/
//!                     |              '-- 32-bit CRC
//!                     '----------------- original message
//!             \--------------+------------/
//!                            '---------- codeword 
//! ```
//!
//! Thanks to testing done by Philip Koopman, we know that CRC32C has a [Hamming
//! distance][hamming-distance] >= 3 up to a message size of 2,147,483,615 bits
//! or 255 MiB ([src][crc-hd]). What this means is it takes at minimum 3 or more
//! bit flips to reach the next valid codeword.
//!
//! But it also means that if there is one or fewer bit-flips, there is a single
//! codeword that is most likely the correct value!
//!
//! Note there is one clearly correct value if there is only 1 bit-flip. 3 bit-flips
//! might get us a valid, but different codeword, and 2-bit flips might give us a
//! codeword _equidistant_ between two other codewords, in which case we can't be
//! sure which codeword is the "most correct".
//!
//! A naive, but completely functional error-correction code, is to simply try all
//! bit flips:
//!
//! ``` rust
//! use ::gf256::crc::crc32c;
//! # use std::convert::TryFrom;
//!
//! fn crc32c_correct(codeword: &[u8]) -> Option<Vec<u8>> {
//!     // try flipping each bit
//!     for i in 0..8*codeword.len() {
//!         let mut codeword = codeword.to_owned();
//!         codeword[i/8] ^= 1 << (8-1-(i % 8));
//!
//!         // found correct crc?
//!         let crc = u32::from_le_bytes(<[u8; 4]>::try_from(
//!             &codeword[codeword.len()-4..]
//!         ).unwrap());
//!
//!         if crc32c(&codeword[..codeword.len()-4], 0) == crc {
//!             return Some(codeword);
//!         }
//!     }
//!
//!     // failed to find the error
//!     None
//! }
//! 
//! // try a few bit-flips
//! assert_eq!(crc32c_correct(b"hgllo!\x5b\xfd\x09\x8c"), Some(b"hello!\x5b\xfd\x09\x8c".to_vec()));
//! assert_eq!(crc32c_correct(b"hello!\xdb\xfd\x09\x8c"), Some(b"hello!\x5b\xfd\x09\x8c".to_vec()));
//! assert_eq!(crc32c_correct(b"hello \x5b\xfd\x09\x8c"), Some(b"hello!\x5b\xfd\x09\x8c".to_vec()));
//! ```
//!
//! We can take this further. CRC32C has a Hamming distance >= 8 up to a message
//! size of 177 bits, or 22 bytes. So in this codeword, we can even correct
//! 4 (8/2) bit-flips:
//!
//! ``` rust
//! use ::gf256::crc::crc32c;
//! # use std::convert::TryFrom;
//!
//! fn crc32c_correct(codeword: &[u8]) -> Option<Vec<u8>> {
//!     // try flipping each bit
//!     for i in 0..8*codeword.len() {
//!      for j in 0..8*codeword.len() {
//!       for k in 0..8*codeword.len() {
//!        for l in 0..8*codeword.len() {
//!         let mut codeword = codeword.to_owned();
//!         codeword[i/8] ^= 1 << (8-1-(i % 8));
//!         codeword[j/8] ^= 1 << (8-1-(j % 8));
//!         codeword[k/8] ^= 1 << (8-1-(k % 8));
//!         codeword[l/8] ^= 1 << (8-1-(l % 8));
//!
//!         // found correct crc?
//!         let crc = u32::from_le_bytes(<[u8; 4]>::try_from(
//!             &codeword[codeword.len()-4..]
//!         ).unwrap());
//!
//!         if crc32c(&codeword[..codeword.len()-4], 0) == crc {
//!             return Some(codeword);
//!         }
//!        }  
//!       }  
//!      }
//!     }
//!
//!     // failed to find the error
//!     None
//! }
//! 
//! // try a few bit-flips
//! assert_eq!(crc32c_correct(b"jgnlo#\x5b\xfd\x09\x8c"), Some(b"hello!\x5b\xfd\x09\x8c".to_vec()));
//! ```
//!
//! But this starts to get very expensive.
//!
//! It's also worth noting that using a CRC as an error-correction code reduces its
//! usefulness as a _checksum_. This is because correcting errors risks changing the
//! codeword to a valid, but different, codeword.
//!
//! ## How does Reed-Solomon error-correction work?
//! 
//! Reed-Solomon error-correction codes are actually very similar to CRCs. They
//! both involve viewing the message as a polynomial, and appending the remainder
//! after polynomial division by a constant.
//!
//! However:
//!
//! 1. Reed-Solomon views the message as a polynomial in a finite-field,
//!    usually `GF(256)`.
//!
//!    So for example:
//!
//!    ``` text
//!    hello!  68 65 6c 6c 6f 21
//!    ``` 
//!
//!    Would be viewed as:
//!
//!    ``` text
//!    f(x) = 68 x^5 + 65 x^4 + 6c x^3 + 6c x^2 + 6f x + 21
//!    ```
//!
//!    Note! We are dealing with polynomials built out of elements in `GF(256)`.
//!    Elements in `GF(256)` are _also_ built out polynomials, but this is
//!    irrelevant for the implementation of Reed-Solomon.
//!
//!    Try to ignore the implementation details of `GF(256)` here, and just
//!    treat it as a set of numbers in a conveniently byte-sized finite-field.
//!
//! 2. The constant polynomial we use as a divisor, called the "generator
//!    polynomial", is chosen to have some very special properties that makes
//!    finding errors much more efficient.
//!
//! So how do we create this special "generator polynomial"?
//!
//! Consider this little polynomial:
//!
//! ``` text
//! f(x) = x - c
//! ```
//!
//! This polynomial has the nice property that it's zero when `x` equals `c`:
//!
//! ``` text
//! f(c) = c - c = 0
//! ```
//!
//! And because multiplication by zero is, well, zero, we can this with
//! any polynomial to give us a new polynomial that is also zero when
//! `x` equals `c`:
//!
//! ``` text
//! f(x) = (x - c)*g(x)
//! f(c) = (c - c)*g(x) = 0*g(x) = 0
//! ```
//!
//! We can even multiply multiple polynomials in this form to create a
//! new polynomial that is zero at any arbitrary set of points:
//!
//! ``` text
//! f(x) = (x - c0)*(x - c1)*(x - c2)
//! f(c0) = (c0 - c0)*(c0 - c1)*(c0 - c2) = 0        *(c0 - c1)*(c0 - c2) = 0
//! f(c1) = (c1 - c0)*(c1 - c1)*(c1 - c2) = (c1 - c0)*0        *(c1 - c2) = 0
//! f(c2) = (c2 - c0)*(c2 - c1)*(c2 - c2) = (c2 - c0)*(c2 - c1)*0         = 0
//! ```
//!
//! We can use a generator element in our field, `g`, as a source of unique
//! constants. Recall that the powers of a generator, sometimes called a
//! primtive element, generate all non-zero elements in a finite-field before
//! looping.
//!
//! For more information on generators, see the documentation in [gf](../gf.rs).
//!
//! With this we can create an arbitrary polynomial that evaluates to zero for
//! a fixed set of points:
//!
//! ``` text
//! G(x) = (x - g^0)*(x - g^1)*(x - g^2)*...
//!
//! or
//!
//! G(x) = ∏ (x - g^i)
//!        i
//! ```
//!
//! This is our "generator polynomial". It's called that because it's generated
//! by a generator elements, not because it actually generates anything, which
//! can be a bit confusing.
//!
//! The main feature of the generator polynomial is that it evaluates to zero
//! at a set of fixed points `g^i`. And, because of math, any polynomial
//! multiplied by our generator polynomial will also evaluate to zero at the
//! set of fixed points `g^i`.
//!
//! ``` text
//! let c(x) = m(x)*G(x)
//! c(g^i) = m(g^i)*G(g^i) = m(g^i)*0 = 0
//! ```
//!
//! And this is one possible way of encoding a Reed-Solomon error-correcting
//! code. But it's a bit messy since we end up obscuring our original message.
//!
//! Instead we like to use a "[systematic][systematic]" encoding, which just means
//! our original message is contained in the codeword. We can do this by padding
//! the original message with `n` zeros, and then subtracting the remainder after
//! polynomial division.
//!
//! Just like CRCs, this creates a polynomial that is a multiple of our generator
//! polynomial:
//!
//! ``` text
//! c(x) = m(x) - (m(x) % G(x))
//! ```
//!
//! Polynomial remainder is very convenient because, thanks to no carry between
//! digits, the remainder will always be one term less than the divisor. And since
//! subtraction in `GF(256)` is xor, this is equivalent to concatenating the
//! original message with the remainder.
//!
//! The important thing is that our codeword, `c(x)`, is now perfectly divisible
//! by our generator polynomial, `G(x)`. This means there is some unimportant
//! polynomial `f(x)` such that `c(x)` = `f(x)*G(x)`. And since `G(x)` evaluates
//! to zero for our fixed points, `c(x)` must also evaluate to zero for our fixed
//! points:
//!
//! ``` text
//! c(g^i) = f(g^i)*G(g^i) = f(g^i)*0 = 0
//! ```
//!
//! But what happens if our codeword contains errors?
//!
//! One way of representing errors is by saying our codeword has been xored with
//! some unknown bytes, or equivalently, it has an unknown polynomial, `e(x)`,
//! added to it:
//!
//! ``` text
//! c'(x) = c(x) + e(x)
//! ```
//!
//! Say we have `v` errors at several positions `j`, with magnitude `Yj`:
//!
//! ``` text
//! e(x) = Y0*x^j0 + Y1*x^j1 + Y2*x^j2 + ...
//!
//! or
//!        v
//! e(x) = Σ Yj*x^j
//!        j
//! ```
//!
//! So our errored codeword can be represented as:
//!
//! ``` text
//!                v
//! c'(x) = c(x) + Σ Yj*x^j
//!                j
//! ```
//!
//! Now here's the interesting thing, check out what happens when we evaluate
//! our errored code at our fixed points:
//!
//! ``` text
//!                    v
//! c'(g^i) = c(g^i) + Σ Yj*(g^i)^j
//!                    j
//!
//!               v
//! c'(g^i) = 0 + Σ Yj*(g^i)^j
//!               j
//!
//!           v
//! c'(g^i) = Σ Yj*(g^i)^j
//!           j
//!
//!           v
//! c'(g^i) = Σ Yj*g^(i*j)
//!           j
//!
//! let Xj = g^j
//!
//!           v
//! c'(g^i) = Σ Yj*Xj^i
//!           j
//! ```
//!
//! The original message just drops out! And we're left with a series of
//! equations that may be solvable.
//!
//! We call the evaluation of our recieved message the "syndromes", `Si`, and
//! the terms that describe the errors the "error locators", `Xj` = `g^j`, and
//! the "error values", `Yj`.
//!
//! ``` text
//!                v
//! Si = c'(g^i) = Σ Yj*Xj^i
//!                j
//! ```
//!
//! If our syndromes are all zero, yay! Our codeword arrived intact. Otherwise
//! we need to perform error-correction, which is equivalent to solving for
//! these unknowns.
//!
//! Note that if we can figure out all `Yj` and `Xj`, we can repair our codeword
//! and extract our original message!
//!
//! ``` text
//!                v
//! c(x) = c'(x) - Σ Yj*x^log_g(Xj)
//!                j
//! ```
//!
//! Fortunately, if we know the locations of the errors, `Xj`, solving for the
//! error values, `Yj`, isn't that bad.
//!
//! We have a set of linearly-independent equations, and a set of unknowns. As
//! long as we have more equations than unknowns we can solve this system of
//! equations for the unknowns `Yj`:
//!
//! ``` text
//! S0 = Y0*X0^0 + Y1*X1^0 + Y2*X2^0 + Y3*X3^0 + ...
//! S1 = Y0*X0^1 + Y1*X1^1 + Y2*X2^1 + Y3*X3^1 + ...
//! S2 = Y0*X0^2 + Y1*X1^2 + Y2*X2^2 + Y3*X3^2 + ...
//! S3 = Y0*X0^3 + Y1*X1^3 + Y2*X2^3 + Y3*X3^3 + ...
//! ...
//!
//! Or in matrix form:
//!
//! [S0]   [X0^0 X1^0 X2^0 X3^0 ...] [Y0]
//! [S1]   [X0^1 X1^1 X2^1 X3^1 ...] [Y1]
//! [S2] = [X0^2 X1^2 X2^2 X3^2 ...] [Y2]
//! [S3]   [X0^3 X1^3 X2^3 X3^3 ...] [Y3]
//! [..]   [...                    ] [..]
//!
//! [Y0]   [X0^0 X1^0 X2^0 X3^0 ...]^-1 [S0]
//! [Y1]   [X0^1 X1^1 X2^1 X3^1 ...]    [S1]
//! [Y2] = [X0^2 X1^2 X2^2 X3^2 ...]    [S2]
//! [Y3]   [X0^3 X1^3 X2^3 X3^3 ...]    [S3]
//! [..]   [...                    ]    [..]
//! ```
//!
//! Note that in order to solve for `v` errors, we need `v` equations. This would
//! need syndromes up to `Sv`. So if we have `n` syndromes, we are limited to
//! repairing up to `n` errors at known locations (these are usually called
//! "erasures").
//!
//! When you don't know the locations of the errors, `Xj`, it gets a bit more
//! tricky.
//!
//! Enter the "error locator polynomial", `Λ(x)`:
//!
//! ``` text
//! Λ(x) = (1 - x*X0)*(1 - x*X1)*(1 - x*X2)*...
//!
//! or
//!        v
//! Λ(x) = ∏ (1 - Xk*x)
//!        k
//! ```
//!
//! This a very special polynomial designed to help, well, locate our errors.
//!
//! It has a number of useful properties:
//!
//! 1. `Λ(Xj^-1)` = `0` for any `Xj^-1`, `j` < `n`
//!
//!    This happens for the same reason `G(g^i)` = `0` in our generator polynomial.
//!    `(1 - Xk*Xj^-1)` evaluates to zero when `j` = `k`, and since multiplying
//!    any polynomial by zero evaluates to zero, the entirety of `Λ(Xj^-1)` reduces
//!    to zero.
//!
//! 2. `Λ(0)` = `1`
//!
//!    This prevents trivial solutions for `Λ(x)` = `0`.
//!
//! 3. `Λ(x)`, when multiplied out, gives us an v-term polynomial with some
//!    coefficients, `Λi`:
//!
//!    ``` text
//!    Λ(x) = (1 - X0*x)*(1 - X1*x)*(1 - X2*x)*...
//!
//!    Λ(x) = (1 - (X0 + X1)*x + X0*X1*x^2)*(1 - X2*x)*...
//!
//!    Λ(x) = (1 - (X0 + X1 + X2)*x + (X0*X1 + X0*X2 + X1*X2)*X2*x^2 - X0*X1*X2*x^3)*...
//!
//!    ... blablablah ...
//!
//!    Λ(x) = 1 + Λ1*x + Λ2*x^2 + Λ3*x^3 + ...
//!
//!    or
//!               v
//!    Λ(x) = 1 + Σ Λk*x^i
//!              i=1
//!    ```
//!
//!    Note that floating `+1` in front of the summation. This term doesn't need
//!    an associated constant, which is what makes this polynomial useful.
//!
//! Consider what happens if we multiply the error locator polynomial, `Λ(x)`, at
//! the fixed points `Xj^-1`, with our syndromes, `Si`. We know `Λ(Xj^-1)` = `0`,
//! so this whole thing must also evaluate to zero:
//!
//! ``` text
//! Si*Λ(Xj^-1) = 0
//!
//! Si*(1 + Λ1*Xj^-1 + Λ2*Xj^-2 + Λ3*Xj^-3 + ...) = 0
//!
//! Si + Si*Λ1*Xj^-1 + Si*Λ2*Xj^-2 + Si*Λ3*Xj^-3 + ... = 0
//! ```
//!
//! Recall that `Si` can be defined in terms of `Yj` and `Xj`, so multiplying
//! `Si` by some constant `Xj^-k`:
//!
//! ``` text
//!      v
//! Si = Σ Yj*Xj^i
//!      i
//!
//!             v
//! Si*Xj^-k = (Σ Yj*Xj^i) * Xj^-k
//!             i
//!
//!            v
//! Si*Xj^-k = Σ Yj*Xj^i*Xj^-k
//!            i
//!
//!            v
//! Si*Xj^-k = Σ Yj*Xj^(i-k)
//!            i
//! ```
//!
//! But wait! That's the definition of a different syndrome, `Si-k`:
//!
//! ``` text
//! Si*Xj^-k = Si-k
//! ```
//!
//! We can substitute this in:
//!
//! ``` text
//! Si + Si*Λ1*Xj^-1 + Si*Λ2*Xj^-2 + Si*Λ3*Xj^-3 + ... = 0
//!
//! Si + Si-1*Λ1 + Si-2*Λ2 + Si-3*Λ3 + ... = 0
//!
//! Si = - Si-1*Λ1 - Si-2*Λ2 - Si-3*Λ3 - ...
//!
//! or
//!        v
//! Si = - Σ Si-j*Λ1
//!        j
//! ```
//!
//! Though note `Si` is only really valid when `i` > `0`, so this equation is
//! only really valid when `i` > `v`. We can shift this so it's valid for
//! any `i` > `0`:
//!
//! ``` text
//! Sv+i = - Sv+i-1*Λ1 - Sv+i-2*Λ2 - Sv+i-3*Λ3 - ...
//!
//! or
//!          v
//! Sv+i = - Σ Sv+i-j*Λ1
//!          j
//! ```
//!
//! At this point in our computation, `Si` is known. So we've ended up with another
//! set of linearly-independent equations! As long as we have more equations than
//! unknowns, `Λj`, we can solve for them. And since `Λj` is directly related to `Xj`,
//! solving for `Λj` will let us solve for our error locations:
//! 
//! ``` text
//! Sv   = - Sv-1*Λ1 - Sv-2*Λ2 - Sv-3*Λ3 - Sv-4*Λ4 - ...
//! Sv+1 = - Sv  *Λ1 - Sv-1*Λ2 - Sv-2*Λ3 - Sv-3*Λ4 - ...
//! Sv+2 = - Sv+1*Λ1 - Sv  *Λ2 - Sv-1*Λ3 - Sv-2*Λ4 - ...
//! Sv+3 = - Sv+2*Λ1 - Sv+1*Λ2 - Sv  *Λ3 - Sv-1*Λ4 - ...
//! ...
//!
//! Or in matrix form:
//!
//! [Sv  ]   [-Sv-1 -Sv-2 -Sv-3 -Sv-4 ...] [Λ1]
//! [Sv+1]   [-Sv   -Sv-1 -Sv-2 -Sv-3 ...] [Λ2]
//! [Sv+2] = [-Sv+1 -Sv   -Sv-1 -Sv-2 ...] [Λ3]
//! [Sv+3]   [-Sv+2 -Sv+1 -Sv   -Sv-1 ...] [Λ4]
//! [... ]   [...                        ] [..]
//!
//! [Λ1]   [-Sv-1 -Sv-2 -Sv-3 -Sv-4 ...]^-1 [Sv  ]
//! [Λ2]   [-Sv   -Sv-1 -Sv-2 -Sv-3 ...]    [Sv+1]
//! [Λ3] = [-Sv+1 -Sv   -Sv-1 -Sv-2 ...]    [Sv+2]
//! [Λ4]   [-Sv+2 -Sv+1 -Sv   -Sv-1 ...]    [Sv+3]
//! [..]   [...                        ]    [... ]
//! ```
//!
//! Note that in order to solve for `v` errors, we need `v` equations. This would
//! need syndromes up to `Sv+v` or `S2v`. So if we have `n` syndromes, we are limited
//! to repairing up to `n/2` errors as unknown locations.
//!
//! It should be noted this form of Reed-Solomon, viewing the message as a
//! polynomial over a finite-field and solving via syndromes, is called a
//! [BCH code][bch]. The original form of Reed-Solomon viewed the message as a set
//! of oversaturated points, much like in [Shamir's secret sharing scheme](../shamir).
//! Because it is easier to decode, BCH view is much more common.
//!
//! ---
//!
//! Let's actually start implementing this thing.
//!
//! Say we had a message we wanted to protect against up to 2 unknown
//! byte errors:
//!
//! ``` text
//! hello!  68 65 6c 6c 6f 21
//! ```
//!
//! Reed-Solomon can repair `n/2` unknown errors for `n` symbols of error
//! correction. So we're going to need 4 bytes of ECC.
//!
//! First we create an `n+1` term generator polynomial, `G(x)`:
//!
//! ``` rust
//! # use ::gf256::*;
//! #
//! /// Multiply two polynomials
//! fn poly_mul(f: &[gf256], g: &[gf256]) -> Vec<gf256> {
//!     let mut r = vec![gf256(0); f.len()+g.len()-1];
//!     for i in 0..f.len() {
//!         for j in 0..g.len() {
//!             let r_len = r.len();
//!             r[r_len-1-(i+j)] += f[f.len()-1-i]*g[g.len()-1-j];
//!         }
//!     }
//!     r
//! }
//!
//! // find our generator polynomial
//! //
//! //        n
//! // G(x) = ∏ (x - g^i)
//! //        i
//! //
//! let mut G = vec![gf256(1)];
//! for i in 0..4 {
//!     G = poly_mul(&G, &[gf256(1), -gf256::GENERATOR.pow(i)]);
//! }
//!
//! assert_eq!(&G, &[gf256(0x01), gf256(0x0f), gf256(0x36), gf256(0x78), gf256(0x40)]);
//! ```
//!
//! So our generator polynomial, `G(x)`, is:
//!
//! ``` text
//! G(x) = 01 x^4 + 0f x^3 + 36 x^2 + 78 x + 40
//! ```
//!
//! Recall the generator polynomial should have zeros at the fixed points `g^i`:
//!
//! ``` rust
//! # use ::gf256::*;
//! # use std::convert::TryFrom;
//! #
//! /// Evaluate a polynomial at a given point
//! fn poly_eval(f: &[gf256], x: gf256) -> gf256 {
//!     let mut r = gf256(0);
//!     for i in 0..f.len() {
//!         r += f[i]*x.pow(u8::try_from(f.len()-1-i).unwrap());
//!     }
//!     r
//! }
//!
//! # let G = [gf256(0x01), gf256(0x0f), gf256(0x36), gf256(0x78), gf256(0x40)];
//! #
//! assert_eq!(poly_eval(&G, gf256::GENERATOR.pow(0)), gf256(0));
//! assert_eq!(poly_eval(&G, gf256::GENERATOR.pow(1)), gf256(0));
//! assert_eq!(poly_eval(&G, gf256::GENERATOR.pow(2)), gf256(0));
//! assert_eq!(poly_eval(&G, gf256::GENERATOR.pow(3)), gf256(0));
//! ```
//! 
//! Now we want to encode our message using `G(x)`. This is done by concatenating
//! the original message with the remainder after polynomial division by `G(x)`:
//!
//! ``` rust
//! # use ::gf256::*;
//! #
//! /// Divide two polynomials using synthetic division
//! fn poly_divrem(f: &[gf256], g: &[gf256]) -> (Vec<gf256>, Vec<gf256>) {
//!     let leading_coeff = g[0];
//!     let mut r = f.to_owned();
//!
//!     for i in 0 .. (f.len() - g.len() + 1) {
//!         r[i] /= leading_coeff;
//!         for j in 1..g.len() {
//!             let r_i = r[i];
//!             r[i+j] -= r_i * g[j];
//!         }
//!     }
//!
//!     let div = r[..f.len()-g.len()+1].to_owned();
//!     let rem = r[f.len()-g.len()+1..].to_owned();
//!     (div, rem)
//! }
//!
//! let message = gf256::slice_from_slice(b"hello!");
//! # let G = [gf256(0x01), gf256(0x0f), gf256(0x36), gf256(0x78), gf256(0x40)];
//!
//! let message_padded = [message, &vec![gf256(0); 4]].concat();
//! let (_, ecc) = poly_divrem(&message_padded, &G);
//! let codeword = [message, &ecc].concat();
//!
//! assert_eq!(&codeword, &[
//!     gf256(0x68), gf256(0x65), gf256(0x6c), gf256(0x6c),
//!     gf256(0x6f), gf256(0x21), gf256(0x15), gf256(0xe5),
//!     gf256(0xab), gf256(0x18)
//! ]);
//! ```
//!
//! So our codeword, `c(x)`, is:
//!
//! ``` text
//! hello!.... 68 65 6c 6c 6f 21 15 e5 ab 18
//!            \-------+-------/ \----+----/
//!                    |              '-- 4-byte ECC
//!                    '----------------- original message
//!            \--------------+------------/
//!                           '---------- codeword 
//! ```
//!
//! Our codeword, `c(x)`, should now be a multiple of `G(x)`. And, since `G(x)`
//! evaluated to zero at the fixed points `g^i`, `c(x)` should also evaluate
//! to zero at the fixed points `g^i`:
//!
//! ``` rust
//! # use ::gf256::*;
//! # use std::convert::TryFrom;
//! #
//! # /// Evaluate a polynomial at a given point
//! # fn poly_eval(f: &[gf256], x: gf256) -> gf256 {
//! #     let mut r = gf256(0);
//! #     for i in 0..f.len() {
//! #         r += f[i]*x.pow(u8::try_from(f.len()-1-i).unwrap());
//! #     }
//! #     r
//! # }
//! #
//! # let codeword = [
//! #     gf256(0x68), gf256(0x65), gf256(0x6c), gf256(0x6c),
//! #     gf256(0x6f), gf256(0x21), gf256(0x15), gf256(0xe5),
//! #     gf256(0xab), gf256(0x18)
//! # ];
//! #
//! assert_eq!(poly_eval(&codeword, gf256::GENERATOR.pow(0)), gf256(0));
//! assert_eq!(poly_eval(&codeword, gf256::GENERATOR.pow(1)), gf256(0));
//! assert_eq!(poly_eval(&codeword, gf256::GENERATOR.pow(2)), gf256(0));
//! assert_eq!(poly_eval(&codeword, gf256::GENERATOR.pow(3)), gf256(0));
//! ```
//!
//! These are our syndromes!
//!
//! ``` rust
//! # use ::gf256::*;
//! # use std::convert::TryFrom;
//! #
//! # /// Evaluate a polynomial at a given point
//! # fn poly_eval(f: &[gf256], x: gf256) -> gf256 {
//! #     let mut r = gf256(0);
//! #     for i in 0..f.len() {
//! #         r += f[i]*x.pow(u8::try_from(f.len()-1-i).unwrap());
//! #     }
//! #     r
//! # }
//! #
//! # let codeword = [
//! #     gf256(0x68), gf256(0x65), gf256(0x6c), gf256(0x6c),
//! #     gf256(0x6f), gf256(0x21), gf256(0x15), gf256(0xe5),
//! #     gf256(0xab), gf256(0x18)
//! # ];
//! #
//! let mut S = vec![gf256(0); 4];
//! for i in 0..S.len() {
//!     S[i] = poly_eval(&codeword, gf256::GENERATOR.pow(u8::try_from(i).unwrap()));
//! }
//!
//! assert_eq!(&S, &[gf256(0), gf256(0), gf256(0), gf256(0)]);
//! ```
//!
//! Let's see what happens if we introduce some errors:
//!
//! ``` text
//! hexlo!x... 68 65 78 6c 6f 21 78 e5 ab 18
//! ```
//!
//! ``` rust
//! # use ::gf256::*;
//! # use std::convert::TryFrom;
//! #
//! # /// Evaluate a polynomial at a given point
//! # fn poly_eval(f: &[gf256], x: gf256) -> gf256 {
//! #     let mut r = gf256(0);
//! #     for i in 0..f.len() {
//! #         r += f[i]*x.pow(u8::try_from(f.len()-1-i).unwrap());
//! #     }
//! #     r
//! # }
//! #
//! let mut codeword = [
//!      gf256(0x68), gf256(0x65), gf256(0x78), gf256(0x6c),
//!      gf256(0x6f), gf256(0x21), gf256(0x78), gf256(0xe5),
//!      gf256(0xab), gf256(0x18)
//! ];
//!
//! let mut S = vec![gf256(0); 4];
//! for i in 0..S.len() {
//!     S[i] = poly_eval(&codeword, gf256::GENERATOR.pow(u8::try_from(i).unwrap()));
//! }
//!
//! assert_eq!(&S, &[gf256(0x79), gf256(0x9d), gf256(0x23), gf256(0xe0)]);
//! ```
//!
//! Our syndromes are no longer zero, which means we've detected some errors.
//!
//! In order to repair these errors, we need to find `v`, the number of errors,
//! `Xj`, the error locations, and `Yj`, the error magnitudes.
//!
//! The first step is to find the number of errors, `v`, and the coefficients of
//! the error locator polynomial, `Λ(x)`:
//!
//! ``` text
//!
//! Λ(x) = 1 + Λ1*x + Λ2*x^2 + Λ3*x^3 + ...
//!
//! or
//!            v
//! Λ(x) = 1 + Σ Λk*x^i
//!           i=1
//! ```
//!
//! An efficient method for finding both of these is the [Berlekamp–Massey
//! algorithm][berlekamp-massey], which iteratively adjusts an estimated
//! error locator polynomial by its discrepancy from the previously found
//! equation:
//!
//! ``` text
//! Sv+i + Sv+i-1*Λ1 + Sv+i-2*Λ2 + Sv+i-3*Λ3 + ... = 0
//! ```
//!
//! Berlekamp–Massey works through the error locator a term at a time, increasing
//! the estimated number of errors until a solution is found:
//!
//! ``` rust
//! # #![allow(mixed_script_confusables)]
//! # use ::gf256::*;
//! #
//! # let S = [gf256(0x79), gf256(0x9d), gf256(0x23), gf256(0xe0)];
//! #
//! /// Multiply a polynomial by a scalar
//! fn poly_scale(f: &[gf256], c: gf256) -> Vec<gf256> {
//!     let mut r = f.to_owned();
//!     for i in 0..r.len() {
//!         r[i] *= c;
//!     }
//!     r
//! }
//!
//! /// Add two polynomials
//! fn poly_add(f: &[gf256], g: &[gf256]) -> Vec<gf256> {
//!     let mut r = f.to_owned();
//!     for i in 0..g.len() {
//!         let r_len = r.len();
//!         r[r_len-1-i] += g[g.len()-1-i];
//!     }
//!     r
//! }
//!
//! // the current estimate for the error locator polynomial
//! let mut Λ = vec![gf256(0), gf256(0), gf256(0), gf256(0), gf256(1)];
//! let mut prev_Λ = Λ.clone();
//!
//! // the current estimate for the number of errors
//! let mut v = 0;
//!
//! for i in 0..S.len() {
//!     let mut delta = S[i];
//!     for j in 1..v+1 {
//!         delta += Λ[Λ.len()-1-j] * S[i-j];
//!     }
//!
//!     // shift
//!     prev_Λ.rotate_left(1);
//!
//!     if delta != gf256(0) {
//!         if 2*v <= i {
//!             let next_Λ = poly_scale(&prev_Λ, delta);
//!             prev_Λ = poly_scale(&Λ, delta.recip());
//!             Λ = next_Λ;
//!             v = i+1-v;
//!         }
//!
//!         Λ = poly_add(&Λ, &poly_scale(&prev_Λ, delta));
//!     }
//! }
//!
//! // trim leading zeros
//! let zeros = Λ.iter().take_while(|x| **x == gf256(0)).count();
//! Λ.drain(0..zeros);
//!
//! assert_eq!(&Λ, &[gf256(0x74), gf256(0x88), gf256(0x01)]);
//! ```
//!
//! That should be the coefficients of our error locator polynomial. Note the
//! extra `+1` which is in the expected equation:
//!
//! ``` text
//! Λ(x) = 74 x^2 + 88 x + 1
//!
//! Λ1 = 88
//! Λ2 = 74
//! ```
//!
//! So now, knowing the formula for the error locator polynomial:
//!
//! ``` text
//!
//! Λ(x) = 1 + Λ1*x + Λ2*x^2 + Λ3*x^3 + ...
//!
//! or
//!            v
//! Λ(x) = 1 + Σ Λk*x^i
//!           i=1
//! ```
//!
//! And that `Λ(Xj^-1)` = `0` for all `Xj`, we just need to find all `x` where
//! `Λ(x)` = `0`, these will be the inverse of our error locations.
//! 
//! Unfortunately this is easier said than done.
//!
//! But we know `Xj` must be a location in our codeword, which, even at the maximum
//! size of a Reed-Solomon codeword in `GF(256)`, really isn't that large. So we
//! can just find the error locations using a brute force search:
//!
//! ``` rust
//! # #![allow(mixed_script_confusables)]
//! # use ::gf256::*;
//! # use std::convert::TryFrom;
//! #
//! # /// Evaluate a polynomial at a given point
//! # fn poly_eval(f: &[gf256], x: gf256) -> gf256 {
//! #     let mut r = gf256(0);
//! #     for i in 0..f.len() {
//! #         r += f[i]*x.pow(u8::try_from(f.len()-1-i).unwrap());
//! #     }
//! #     r
//! # }
//! #
//! # let Λ = [gf256(0x74), gf256(0x88), gf256(0x01)];
//! # let codeword = [
//! #      gf256(0x68), gf256(0x65), gf256(0x78), gf256(0x6c),
//! #      gf256(0x6f), gf256(0x21), gf256(0x78), gf256(0xe5),
//! #      gf256(0xab), gf256(0x18)
//! # ];
//! #
//! let mut error_locations = vec![];
//! for j in 0..codeword.len() {
//!     let Xj = gf256::GENERATOR.pow(u8::try_from(codeword.len()-1-j).unwrap());
//!     let zero = poly_eval(&Λ, Xj.recip());
//!     if zero == gf256(0) {
//!         // found an error location!
//!         error_locations.push(j);
//!     }
//! }
//!
//! assert_eq!(&error_locations, &[2, 6]);
//! ```
//!
//! Note this is only `O(n)`, unlike the `O(n^m)` brute force search we did for CRCs.
//!
//! And sure enough, there's our error locations!
//!
//! Now that we know where the errors are, we just need to find the error
//! magnitudes, `Yj`, for each location.
//!
//! An efficient method for finding the error magnitudes is [Forney's algorithm][forney].
//! The math is beyond me, but it gives us a relatively straightforward formula
//! for each error magnitude:
//!
//! ``` text
//!        Xj*Ω(Xj^-1)
//! Yj = - -----------
//!         Λ'(Xj^-1)
//! ```
//!
//! Where `Ω(x)` is the "error evaluator polynomial" defined as:
//!
//! ``` text
//! Ω(x) = S(x)*Λ(x) mod x^2v
//! ```
//!
//! And `S(x)` is the "partial syndrome polynomial" defined as:
//!
//! ``` text
//! S(x) = S0 + S1*x + S2*x^2 + ...
//!
//! or
//!       2v
//! S(x) = Σ Si*x^i
//!        i
//! ```
//!
//! And `Λ'(x)` is the "[formal derivative][formal-derivative]" of `Λ(x)`,
//! defined as:
//!
//! ``` text
//! Λ'(x) = Λ1 + 2*Λ2*x + 3*Λ3*x^2 + ...
//!
//! or
//!         v
//! Λ'(x) = Σ i*Λi*x^(i-1)
//!        i=1
//! ```
//!
//! Note that `i` here is not a finite-field element! The multiplication between
//! `i` and the field elements is actually repeated addition, not normal
//! finite-field multiplication.
//! 
//! ``` rust
//! # #![allow(mixed_script_confusables)]
//! # use ::gf256::*;
//! # use std::convert::TryFrom;
//! #
//! # /// Multiply two polynomials
//! # fn poly_mul(f: &[gf256], g: &[gf256]) -> Vec<gf256> {
//! #     let mut r = vec![gf256(0); f.len()+g.len()-1];
//! #     for i in 0..f.len() {
//! #         for j in 0..g.len() {
//! #             let r_len = r.len();
//! #             r[r_len-1-(i+j)] += f[f.len()-1-i]*g[g.len()-1-j];
//! #         }
//! #     }
//! #     r
//! # }
//! #
//! # fn poly_eval(f: &[gf256], x: gf256) -> gf256 {
//! #     let mut r = gf256(0);
//! #     for i in 0..f.len() {
//! #         r += f[i]*x.pow(u8::try_from(f.len()-1-i).unwrap());
//! #     }
//! #     r
//! # }
//! #
//! # let S = [gf256(0x79), gf256(0x9d), gf256(0x23), gf256(0xe0)];
//! # let Λ = [gf256(0x74), gf256(0x88), gf256(0x01)];
//! # let error_locations = [2, 6];
//! # let codeword = [
//! #      gf256(0x68), gf256(0x65), gf256(0x78), gf256(0x6c),
//! #      gf256(0x6f), gf256(0x21), gf256(0x78), gf256(0xe5),
//! #      gf256(0xab), gf256(0x18)
//! # ];
//! #
//! // find the erasure evaluator polynomial
//! //
//! // Ω(x) = S(x)*Λ(x) mod x^2v
//! //
//! let mut S_rev = S.clone();
//! S_rev.reverse();
//! let mut Ω = poly_mul(&S_rev, &Λ);
//! Ω.drain(..Ω.len()-S.len());
//!
//! // find the formal derivative of Λ
//! //
//! // Λ'(x) = Σ i*Λi*x^(i-1)
//! //        i=1
//! //
//! let mut Λ_prime = vec![gf256(0); Λ.len()-1];
//! for i in 1..Λ.len() {
//!     let mut sum = gf256(0);
//!     for _ in 0..i {
//!         sum += Λ[Λ.len()-1-i];
//!     }
//!     let Λ_prime_len = Λ_prime.len();
//!     Λ_prime[Λ_prime_len-1-(i-1)] = sum;
//! }
//!
//! // find the error magnitudes
//! //
//! //        Xj*Ω(Xj^-1)
//! // Yj = - -----------
//! //         Λ'(Xj^-1)
//! //
//! let mut error_magnitudes = vec![];
//! for j in error_locations {
//!     let Xj = gf256::GENERATOR.pow(u8::try_from(codeword.len()-1-j).unwrap());
//!     let Yj = -Xj*poly_eval(&Ω, Xj.recip()) / poly_eval(&Λ_prime, Xj.recip());
//!     error_magnitudes.push(Yj);
//! }
//!
//! assert_eq!(&error_magnitudes, &[gf256(0x14), gf256(0x6d)]);
//! ```
//!
//! And now, all we need to do is subtract our error magnitudes from the modified
//! codeword in order to recover our original message!
//!
//! ``` rust
//! # use ::gf256::*;
//! #
//! # let error_locations = [2, 6];
//! # let error_magnitudes = [gf256(0x14), gf256(0x6d)];
//! # let mut codeword = [
//! #      gf256(0x68), gf256(0x65), gf256(0x78), gf256(0x6c),
//! #      gf256(0x6f), gf256(0x21), gf256(0x78), gf256(0xe5),
//! #      gf256(0xab), gf256(0x18)
//! # ];
//! #
//! for i in 0..error_locations.len() {
//!     codeword[error_locations[i]] -= error_magnitudes[i];
//! }
//!
//! assert_eq!(&codeword, &[
//!     gf256(0x68), gf256(0x65), gf256(0x6c), gf256(0x6c),
//!     gf256(0x6f), gf256(0x21), gf256(0x15), gf256(0xe5),
//!     gf256(0xab), gf256(0x18)
//! ]);
//! ```
//!
//! And there we have it! The recovered codeword is:
//!
//! ``` text
//! hello!.... 68 65 6c 6c 6f 21 15 e5 ab 18
//! ```
//!
//! ## Limitations
//!
//! In order for Reed-Solomon to work, we need a unique non-zero error
//! location, `Xj`, for each symbol in our codeword. This limits the size of the
//! _total_ codeword, the message + ecc, to the number of non-zero elements in
//! the field. In the case of `GF(256)`, this limits Reed-Solomon to 255-byte
//! codewords.
//!
//! The most common scheme is 32 bytes of ECC with up to 223 bytes of message,
//! provided by this crate as [`rs255w223`](crate::rs::rs255w223). This was the
//! scheme famously used on the [Voyager missions][voyager].
//!
//! ## Further reading
//!
//! Reed-Solomon error-correction, and error-correction in general, is a deep
//! and complex field. The encoder/decoder presented here is just one method of
//! encoding/decoding for one representation of Reed-Solomon error-correction.
//!
//! This documentation is the aggregation of knowledge from a number of helpful
//! sources that contain more in-depth information:
//!
//! - [Wikipedia][rs-wiki]
//! - [Wikiversity][rs-wikiversity]
//! - [John Gill's lecture notes][rs-gill]
//! - [Henry D. Pfister's Algebraic Decoding of Reed-Solomon and BCH Codes][rs-pfister]
//! 
//!
//! [rs-wiki]: https://en.wikipedia.org/wiki/Reed%E2%80%93Solomon_error_correction
//! [wespa]: https://www.wespa.org/csw19ik.pdf
//! [hamming-distance]: https://en.wikipedia.org/wiki/Hamming_distance
//! [crc-hd]: https://users.ece.cmu.edu/~koopman/crc
//! [systematic]: https://en.wikipedia.org/wiki/Systematic_code
//! [bch]: https://en.wikipedia.org/wiki/BCH_code
//! [forney]: https://en.wikipedia.org/wiki/Forney_algorithm
//! [formal-derivative]: https://en.wikipedia.org/wiki/Formal_derivative
//! [voyager]: https://en.wikipedia.org/wiki/Voyager_program
//! [rs-wikiversity]: https://en.wikiversity.org/wiki/Reed%E2%80%93Solomon_codes_for_coders
//! [rs-gill]: https://web.archive.org/web/20140630172526/http://web.stanford.edu/class/ee387/handouts/notes7.pdf
//! [rs-pfister]: http://pfister.ee.duke.edu/courses/ecen604/rsdecode.pdf
//! [rs-example]: https://github.com/geky/gf256/blob/master/examples/rs.rs


/// A macro for generating custom Reed-Solomon error-correction modules.
///
/// ``` rust,ignore
/// # use ::gf256::*;
/// # use ::gf256::rs::rs;
/// #[rs(block=255, data=223)]
/// pub mod my_rs255w223 {}
///
/// # fn main() -> Result<(), my_rs255w223::Error> {
/// // encode
/// let mut buf = b"Hello World!".to_vec();
/// buf.resize(buf.len()+32, 0u8);
/// my_rs255w223::encode(&mut buf);
/// 
/// // corrupt
/// buf[0..16].fill(b'x');
/// 
/// // correct
/// my_rs255w223::correct_errors(&mut buf)?;
/// assert_eq!(&buf[0..12], b"Hello World!");
/// # Ok::<(), my_rs255w223::Error>(())
/// # }
/// ```
///
/// The `rs` macro accepts a number of configuration options:
///
/// - `block` - Size of the codeword, data+ecc, in bytes.
/// - `data` - Maximum size of the data in bytes.
/// - `gf` - The finite-field we are implemented over, defaults to
///   [`gf256`](crate::gf256).
/// - `u` - The unsigned type to operate on, defaults to [`u8`].
///
/// ``` rust,ignore
/// # use ::gf256::*;
/// # use ::gf256::rs::rs;
/// #[rs(
///     block=255,
///     data=223,
///     gf=gf256,
///     u=u8,
/// )]
/// pub mod my_rs255w223 {}
///
/// # fn main() -> Result<(), my_rs255w223::Error> {
/// // encode
/// let mut buf = b"Hello World!".to_vec();
/// buf.resize(buf.len()+32, 0u8);
/// my_rs255w223::encode(&mut buf);
/// 
/// // corrupt
/// buf[0..16].fill(b'x');
/// 
/// // correct
/// my_rs255w223::correct_errors(&mut buf)?;
/// assert_eq!(&buf[0..12], b"Hello World!");
/// # Ok::<(), my_rs255w223::Error>(())
/// # }
/// ```
///
pub use gf256_macros::rs;


// Reed-Solomon error-correction functions
//
#[rs(block=255, data=223)]
pub mod rs255w223 {}


#[cfg(test)]
mod test {
    use super::*;
    use crate::gf::*;

    extern crate alloc;
    use alloc::vec::Vec;

    // a smaller Reed-Solomon code
    #[rs(block=26, data=16)]
    pub mod rs26w16 {}

    #[test]
    fn rs26w16() {
        let mut data = (0..26).collect::<Vec<u8>>();
        rs26w16::encode(&mut data);
        assert!(rs26w16::is_correct(&data));

        // correct up to k known erasures
        for i in 0..(26-16) {
            data[0..i].fill(b'x');
            let res = rs26w16::correct_erasures(&mut data, &(0..i).collect::<Vec<_>>());
            assert_eq!(res.ok(), Some(i));
            assert_eq!(&data[0..16], &(0..16).collect::<Vec<u8>>());
        }

        // correct up to k/2 unknown errors
        for i in 0..(26-16)/2 {
            data[0..i].fill(b'x');
            let res = rs26w16::correct_errors(&mut data);
            assert_eq!(res.ok(), Some(i));
            assert_eq!(&data[0..16], &(0..16).collect::<Vec<u8>>());
        }
    }

    #[test]
    fn rs26w16_any() {
        let mut data = (0..26).collect::<Vec<u8>>();
        rs26w16::encode(&mut data);

        // try any single error
        for i in 0..26 {
            data[i] = b'x';
            let res = rs26w16::correct_errors(&mut data);
            assert_eq!(res.ok(), Some(1));
            assert_eq!(&data[0..16], &(0..16).collect::<Vec<u8>>());
        }
    }

    #[test]
    fn rs26w16_burst() {
        let mut data = (0..26).collect::<Vec<u8>>();
        rs26w16::encode(&mut data);

        // try any burst of k/2 errors
        for i in 0..26-((26-16)/2) {
            data[i..i+((26-16)/2)].fill(b'x');
            let res = rs26w16::correct_errors(&mut data);
            assert_eq!(res.ok(), Some((26-16)/2));
            assert_eq!(&data[0..16], &(0..16).collect::<Vec<u8>>());
        }
    }

    #[test]
    fn rs255w223() {
        let mut data = (0..255).collect::<Vec<u8>>();
        rs255w223::encode(&mut data);
        assert!(rs255w223::is_correct(&data));

        // correct up to k known erasures
        for i in 0..(255-223) {
            data[0..i].fill(b'x');
            let res = rs255w223::correct_erasures(&mut data, &(0..i).collect::<Vec<_>>());
            assert_eq!(res.ok(), Some(i));
            assert_eq!(&data[0..223], &(0..223).collect::<Vec<u8>>());
        }

        // correct up to k/2 unknown errors
        for i in 0..(255-223)/2 {
            data[0..i].fill(b'x');
            let res = rs255w223::correct_errors(&mut data);
            assert_eq!(res.ok(), Some(i));
            assert_eq!(&data[0..223], &(0..223).collect::<Vec<u8>>());
        }
    }

    #[test]
    fn rs255w223_any() {
        let mut data = (0..255).collect::<Vec<u8>>();
        rs255w223::encode(&mut data);

        // try any single error
        for i in 0..255 {
            data[i] = b'\xff';
            let res = rs255w223::correct_errors(&mut data);
            assert_eq!(res.ok(), Some(1));
            assert_eq!(&data[0..223], &(0..223).collect::<Vec<u8>>());
        }
    }

    #[test]
    fn rs255w223_burst() {
        let mut data = (0..255).collect::<Vec<u8>>();
        rs255w223::encode(&mut data);

        // try any burst of k/2 errors
        for i in 0..255-((255-223)/2) {
            data[i..i+((255-223)/2)].fill(b'\xff');
            let res = rs255w223::correct_errors(&mut data);
            assert_eq!(res.ok(), Some((255-223)/2));
            assert_eq!(&data[0..223], &(0..223).collect::<Vec<u8>>());
        }
    }

    // try a shortened message
    #[test]
    fn rs255w223_shortened() {
        let mut data = (0..40).collect::<Vec<u8>>();
        rs255w223::encode(&mut data);
        assert!(rs255w223::is_correct(&data));

        // correct up to k known erasures
        for i in 0..(40-8) {
            data[0..i].fill(b'x');
            let res = rs255w223::correct_erasures(&mut data, &(0..i).collect::<Vec<_>>());
            assert_eq!(res.ok(), Some(i));
            assert_eq!(&data[0..8], &(0..8).collect::<Vec<u8>>());
        }

        // correct up to k/2 unknown errors
        for i in 0..(40-8)/2 {
            data[0..i].fill(b'x');
            let res = rs255w223::correct_errors(&mut data);
            assert_eq!(res.ok(), Some(i));
            assert_eq!(&data[0..8], &(0..8).collect::<Vec<u8>>());
        }
    }

    // try an overly saturated RS scheme
    #[rs(block=64, data=8)]
    mod rs64w8 {}

    #[test]
    fn rs64w8() {
        let mut data = (0..64).collect::<Vec<u8>>();
        rs64w8::encode(&mut data);
        assert!(rs64w8::is_correct(&data));

        // correct up to k known erasures
        for i in 0..(64-8) {
            data[0..i].fill(b'x');
            let res = rs64w8::correct_erasures(&mut data, &(0..i).collect::<Vec<_>>());
            assert_eq!(res.ok(), Some(i));
            assert_eq!(&data[0..8], &(0..8).collect::<Vec<u8>>());
        }

        // correct up to k/2 unknown errors
        for i in 0..(64-8)/2 {
            data[0..i].fill(b'x');
            let res = rs64w8::correct_errors(&mut data);
            assert_eq!(res.ok(), Some(i));
            assert_eq!(&data[0..8], &(0..8).collect::<Vec<u8>>());
        }
    }

    // multi-byte Reed-Solomon
    #[rs(gf=gf2p64, u=u64, block=26, data=16)]
    pub mod gf2p64_rs26w16 {}

    #[test]
    fn gf2p64_rs26w16() {
        let mut data = (0..26).collect::<Vec<u64>>();
        gf2p64_rs26w16::encode(&mut data);
        assert!(gf2p64_rs26w16::is_correct(&data));

        // correct up to k known erasures
        for i in 0..(26-16) {
            data[0..i].fill(0x7878787878787878);
            let res = gf2p64_rs26w16::correct_erasures(&mut data, &(0..i).collect::<Vec<_>>());
            assert_eq!(res.ok(), Some(i));
            assert_eq!(&data[0..16], &(0..16).collect::<Vec<u64>>());
        }

        // correct up to k/2 unknown errors
        for i in 0..(26-16)/2 {
            data[0..i].fill(0x7878787878787878);
            let res = gf2p64_rs26w16::correct_errors(&mut data);
            assert_eq!(res.ok(), Some(i));
            assert_eq!(&data[0..16], &(0..16).collect::<Vec<u64>>());
        }
    }

    // Reed-Solomon with very odd sizes
    #[gf(polynomial=0x13, generator=0x2)]
    type gf16;
    #[rs(gf=gf16, u=u8, block=15, data=8)]
    pub mod gf16_rs15w8 {}
    #[gf(polynomial=0x800021, generator=0x2)]
    type gf2p23;
    #[rs(gf=gf2p23, u=u32, block=26, data=16)]
    pub mod gf2p23_rs26w16 {}

    #[test]
    fn gf2p16_rs15w8() {
        let mut data = (0..15).collect::<Vec<u8>>();
        gf16_rs15w8::encode(&mut data);
        assert!(gf16_rs15w8::is_correct(&data));

        // correct up to k known erasures
        for i in 0..(15-8) {
            data[0..i].fill(0x7);
            let res = gf16_rs15w8::correct_erasures(&mut data, &(0..i).collect::<Vec<_>>());
            assert_eq!(res.ok(), Some(i));
            assert_eq!(&data[0..8], &(0..8).collect::<Vec<u8>>());
        }

        // correct up to k/2 unknown errors
        for i in 0..(15-8)/2 {
            data[0..i].fill(0x7);
            let res = gf16_rs15w8::correct_errors(&mut data);
            assert_eq!(res.ok(), Some(i));
            assert_eq!(&data[0..8], &(0..8).collect::<Vec<u8>>());
        }
    }

    #[test]
    fn gf2p23_rs26w16() {
        let mut data = (0..26).collect::<Vec<u32>>();
        gf2p23_rs26w16::encode(&mut data);
        assert!(gf2p23_rs26w16::is_correct(&data));

        // correct up to k known erasures
        for i in 0..(26-16) {
            data[0..i].fill(0x787878);
            let res = gf2p23_rs26w16::correct_erasures(&mut data, &(0..i).collect::<Vec<_>>());
            assert_eq!(res.ok(), Some(i));
            assert_eq!(&data[0..16], &(0..16).collect::<Vec<u32>>());
        }

        // correct up to k/2 unknown errors
        for i in 0..(26-16)/2 {
            data[0..i].fill(0x787878);
            let res = gf2p23_rs26w16::correct_errors(&mut data);
            assert_eq!(res.ok(), Some(i));
            assert_eq!(&data[0..16], &(0..16).collect::<Vec<u32>>());
        }
    }

    // all RS params
    #[rs(gf=gf256, u=u8, block=26, data=16)]
    mod rs26w16_all_params {}

    #[test]
    fn rs_all_params() {
        let mut data = (0..26).collect::<Vec<u8>>();
        rs26w16_all_params::encode(&mut data);
        assert!(rs26w16_all_params::is_correct(&data));

        // correct up to k known erasures
        for i in 0..(26-16) {
            data[0..i].fill(b'x');
            let res = rs26w16_all_params::correct_erasures(&mut data, &(0..i).collect::<Vec<_>>());
            assert_eq!(res.ok(), Some(i));
            assert_eq!(&data[0..16], &(0..16).collect::<Vec<u8>>());
        }

        // correct up to k/2 unknown errors
        for i in 0..(26-16)/2 {
            data[0..i].fill(b'x');
            let res = rs26w16_all_params::correct_errors(&mut data);
            assert_eq!(res.ok(), Some(i));
            assert_eq!(&data[0..16], &(0..16).collect::<Vec<u8>>());
        }
    }
}
