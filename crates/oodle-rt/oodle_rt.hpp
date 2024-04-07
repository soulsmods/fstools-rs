#ifndef _FSTOOLS_OODLE_RT
#define _FSTOOLS_OODLE_RT

#include <typeinfo>

#include "oodle2.h"

using Function_OodleLZDecoder_Create = decltype(OodleLZDecoder_Create);
using Function_OodleLZDecoder_Destroy = decltype(OodleLZDecoder_Destroy);
using Function_OodleLZDecoder_DecodeSome = decltype(OodleLZDecoder_DecodeSome);
using Function_OodleLZ_Decompress = decltype(OodleLZ_Decompress);

#endif