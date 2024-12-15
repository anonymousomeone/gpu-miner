pub mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        src: r"
            #version 450

            layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

            layout(set = 0, binding = 0) buffer input_buffer {
                uint input_data[10];
            };

            layout(set = 0, binding = 1) buffer output_buffer {
                uint output_data[];
            };

            #define ROTL(x, n) ((x << n) | (x >> (32 - n)))

            uvec4 add(uvec4 data, uint nonce) {
                uint digit = 0;

                digit = (nonce / 1) % 10;
                data.w += digit;
                digit = (nonce / 10) % 10;
                data.w += digit << 8;
                digit = (nonce / 100) % 10;
                data.w += digit << 16;
                digit = (nonce / 1000) % 10;
                data.w += digit << 24;

                digit = (nonce / 10000) % 10;
                data.z += digit;
                digit = (nonce / 100000) % 10;
                data.z += digit << 8;
                digit = (nonce / 1000000) % 10;
                data.z += digit << 16;
                digit = (nonce / 10000000) % 10;
                data.z += digit << 24;

                digit = (nonce / 100000000) % 10;
                data.y += digit;
                digit = (nonce / 1000000000) % 10;
                data.y += digit << 8;

                return data;
            }

            const uint h0 = 0x67452301;
            const uint h1 = 0xefcdab89;
            const uint h2 = 0x98badcfe;
            const uint h3 = 0x10325476;
            const uint h4 = 0xc3d2e1f0;

            const uint k0 = 0x5a827999;
            const uint k1 = 0x6ed9eba1;
            const uint k2 = 0x8f1bbcdc;
            const uint k3 = 0xca62c1d6;

            void main() {
                uint invocationID = gl_GlobalInvocationID.x;
                uint words[80];

                for (int i = 6; i < 15; i++) {
                    words[i] = 0;
                }

                uvec4 res = add(uvec4(input_data[6], input_data[7], input_data[8], input_data[9]), invocationID);
                words[0] = input_data[5];
                words[1] = res[0];
                words[2] = res[1];
                words[3] = res[2];
                words[4] = res[3];
                words[5] = 0x80000000;
                words[15] = 672;
                // words[4] = input_data[4];
                // words[5] = input_data[5] + gl_GlobalInvocationID.x;
                // words[6] = 0x80000000; // padding
                // words[15] = 192; // message length

                for (int i = 16; i < 80; i++) {
                    words[i] = ROTL((words[i - 3] ^ words[i - 8] ^ words[i - 14] ^ words[i - 16]), 1);    
                }

                // // goofy ahh wikipedia optimization???
                // for (int i = 32; i < 80; i++) {
                //     words[i] = ROTL((words[i - 6] ^ words[i - 16] ^ words[i - 28] ^ words[i - 32]), 2);
                // }

                uint a, b, c, d, e, temp, f, k;
                a = input_data[0];
                b = input_data[1];
                c = input_data[2];
                d = input_data[3];
                e = input_data[4];

                for (int i = 0; i < 80; i++) {
                    if (i < 20) {
                        f = (b & c) | ((~b) & d);
                        k = k0;
                    } else if (i < 40) {
                        f = b ^ c ^ d;
                        k = k1;
                    } else if (i < 60) {
                        f = (b & c) | (b & d) | (c & d);
                        k = k2;
                    } else {
                        f = b ^ c ^ d;
                        k = k3;
                    }
                
                    temp = ROTL(a, 5) + f + e + k + words[i];
                    e = d;
                    d = c;
                    c = ROTL(b, 30);
                    b = a;
                    a = temp;
                }

                uint offset = invocationID * 6;
                output_data[offset] = input_data[0] + a;
                output_data[offset + 1] = input_data[1] + b;
                output_data[offset + 2] = input_data[2] + c;
                output_data[offset + 3] = input_data[3] + d;
                output_data[offset + 4] = input_data[4] + e;
                output_data[offset + 5] = invocationID;
            }
        ",
    }
}

// optimized compute shader; loops unrolled
pub mod ocs {
    vulkano_shaders::shader! {
        ty: "compute",
        src: r"
            #version 450

            layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

            layout(set = 0, binding = 0) buffer input_buffer {
                uint input_data[10];
            };

            layout(set = 0, binding = 1) buffer output_buffer {
                uint output_data[];
            };

            #define ROTL(x, n) ((x << n) | (x >> (32 - n)))

            uvec4 add(uvec4 data, uint nonce) {
                uint digit = 0;

                digit = (nonce / 1) % 10;
                data.w += digit;
                digit = (nonce / 10) % 10;
                data.w += digit << 8;
                digit = (nonce / 100) % 10;
                data.w += digit << 16;
                digit = (nonce / 1000) % 10;
                data.w += digit << 24;

                digit = (nonce / 10000) % 10;
                data.z += digit;
                digit = (nonce / 100000) % 10;
                data.z += digit << 8;
                digit = (nonce / 1000000) % 10;
                data.z += digit << 16;
                digit = (nonce / 10000000) % 10;
                data.z += digit << 24;

                digit = (nonce / 100000000) % 10;
                data.y += digit;
                digit = (nonce / 1000000000) % 10;
                data.y += digit << 8;

                return data;
            }

            const uint h0 = 0x67452301;
            const uint h1 = 0xefcdab89;
            const uint h2 = 0x98badcfe;
            const uint h3 = 0x10325476;
            const uint h4 = 0xc3d2e1f0;

            const uint k0 = 0x5a827999;
            const uint k1 = 0x6ed9eba1;
            const uint k2 = 0x8f1bbcdc;
            const uint k3 = 0xca62c1d6;

            void main() {
                uint invocationID = gl_GlobalInvocationID.x;
                uint words[80];

                uvec4 res = add(uvec4(input_data[6], input_data[7], input_data[8], input_data[9]), invocationID);
                words[0] = input_data[5];
                words[1] = res[0];
                words[2] = res[1];
                words[3] = res[2];
                words[4] = res[3];
                words[5] = 0x80000000;
                words[6] = 0;
                words[7] = 0;
                words[8] = 0;
                words[9] = 0;
                words[10] = 0;
                words[11] = 0;
                words[12] = 0;
                words[13] = 0;
                words[14] = 0;
                words[15] = 672;

                words[16] = ROTL((words[13] ^ words[8] ^ words[2] ^ words[0]), 1);
                words[17] = ROTL((words[14] ^ words[9] ^ words[3] ^ words[1]), 1);
                words[18] = ROTL((words[15] ^ words[10] ^ words[4] ^ words[2]), 1);
                words[19] = ROTL((words[16] ^ words[11] ^ words[5] ^ words[3]), 1);
                words[20] = ROTL((words[17] ^ words[12] ^ words[6] ^ words[4]), 1);
                words[21] = ROTL((words[18] ^ words[13] ^ words[7] ^ words[5]), 1);
                words[22] = ROTL((words[19] ^ words[14] ^ words[8] ^ words[6]), 1);
                words[23] = ROTL((words[20] ^ words[15] ^ words[9] ^ words[7]), 1);
                words[24] = ROTL((words[21] ^ words[16] ^ words[10] ^ words[8]), 1);
                words[25] = ROTL((words[22] ^ words[17] ^ words[11] ^ words[9]), 1);
                words[26] = ROTL((words[23] ^ words[18] ^ words[12] ^ words[10]), 1);
                words[27] = ROTL((words[24] ^ words[19] ^ words[13] ^ words[11]), 1);
                words[28] = ROTL((words[25] ^ words[20] ^ words[14] ^ words[12]), 1);
                words[29] = ROTL((words[26] ^ words[21] ^ words[15] ^ words[13]), 1);
                words[30] = ROTL((words[27] ^ words[22] ^ words[16] ^ words[14]), 1);
                words[31] = ROTL((words[28] ^ words[23] ^ words[17] ^ words[15]), 1);
                words[32] = ROTL((words[29] ^ words[24] ^ words[18] ^ words[16]), 1);
                words[33] = ROTL((words[30] ^ words[25] ^ words[19] ^ words[17]), 1);
                words[34] = ROTL((words[31] ^ words[26] ^ words[20] ^ words[18]), 1);
                words[35] = ROTL((words[32] ^ words[27] ^ words[21] ^ words[19]), 1);
                words[36] = ROTL((words[33] ^ words[28] ^ words[22] ^ words[20]), 1);
                words[37] = ROTL((words[34] ^ words[29] ^ words[23] ^ words[21]), 1);
                words[38] = ROTL((words[35] ^ words[30] ^ words[24] ^ words[22]), 1);
                words[39] = ROTL((words[36] ^ words[31] ^ words[25] ^ words[23]), 1);
                words[40] = ROTL((words[37] ^ words[32] ^ words[26] ^ words[24]), 1);
                words[41] = ROTL((words[38] ^ words[33] ^ words[27] ^ words[25]), 1);
                words[42] = ROTL((words[39] ^ words[34] ^ words[28] ^ words[26]), 1);
                words[43] = ROTL((words[40] ^ words[35] ^ words[29] ^ words[27]), 1);
                words[44] = ROTL((words[41] ^ words[36] ^ words[30] ^ words[28]), 1);
                words[45] = ROTL((words[42] ^ words[37] ^ words[31] ^ words[29]), 1);
                words[46] = ROTL((words[43] ^ words[38] ^ words[32] ^ words[30]), 1);
                words[47] = ROTL((words[44] ^ words[39] ^ words[33] ^ words[31]), 1);
                words[48] = ROTL((words[45] ^ words[40] ^ words[34] ^ words[32]), 1);
                words[49] = ROTL((words[46] ^ words[41] ^ words[35] ^ words[33]), 1);
                words[50] = ROTL((words[47] ^ words[42] ^ words[36] ^ words[34]), 1);
                words[51] = ROTL((words[48] ^ words[43] ^ words[37] ^ words[35]), 1);
                words[52] = ROTL((words[49] ^ words[44] ^ words[38] ^ words[36]), 1);
                words[53] = ROTL((words[50] ^ words[45] ^ words[39] ^ words[37]), 1);
                words[54] = ROTL((words[51] ^ words[46] ^ words[40] ^ words[38]), 1);
                words[55] = ROTL((words[52] ^ words[47] ^ words[41] ^ words[39]), 1);
                words[56] = ROTL((words[53] ^ words[48] ^ words[42] ^ words[40]), 1);
                words[57] = ROTL((words[54] ^ words[49] ^ words[43] ^ words[41]), 1);
                words[58] = ROTL((words[55] ^ words[50] ^ words[44] ^ words[42]), 1);
                words[59] = ROTL((words[56] ^ words[51] ^ words[45] ^ words[43]), 1);
                words[60] = ROTL((words[57] ^ words[52] ^ words[46] ^ words[44]), 1);
                words[61] = ROTL((words[58] ^ words[53] ^ words[47] ^ words[45]), 1);
                words[62] = ROTL((words[59] ^ words[54] ^ words[48] ^ words[46]), 1);
                words[63] = ROTL((words[60] ^ words[55] ^ words[49] ^ words[47]), 1);
                words[64] = ROTL((words[61] ^ words[56] ^ words[50] ^ words[48]), 1);
                words[65] = ROTL((words[62] ^ words[57] ^ words[51] ^ words[49]), 1);
                words[66] = ROTL((words[63] ^ words[58] ^ words[52] ^ words[50]), 1);
                words[67] = ROTL((words[64] ^ words[59] ^ words[53] ^ words[51]), 1);
                words[68] = ROTL((words[65] ^ words[60] ^ words[54] ^ words[52]), 1);
                words[69] = ROTL((words[66] ^ words[61] ^ words[55] ^ words[53]), 1);
                words[70] = ROTL((words[67] ^ words[62] ^ words[56] ^ words[54]), 1);
                words[71] = ROTL((words[68] ^ words[63] ^ words[57] ^ words[55]), 1);
                words[72] = ROTL((words[69] ^ words[64] ^ words[58] ^ words[56]), 1);
                words[73] = ROTL((words[70] ^ words[65] ^ words[59] ^ words[57]), 1);
                words[74] = ROTL((words[71] ^ words[66] ^ words[60] ^ words[58]), 1);
                words[75] = ROTL((words[72] ^ words[67] ^ words[61] ^ words[59]), 1);
                words[76] = ROTL((words[73] ^ words[68] ^ words[62] ^ words[60]), 1);
                words[77] = ROTL((words[74] ^ words[69] ^ words[63] ^ words[61]), 1);
                words[78] = ROTL((words[75] ^ words[70] ^ words[64] ^ words[62]), 1);
                words[79] = ROTL((words[76] ^ words[71] ^ words[65] ^ words[63]), 1);

                uint a, b, c, d, e, temp;
                a = h0;
                b = h1;
                c = h2;
                d = h3;
                e = h4;

                temp = ROTL(a, 5)+((b&c)|((~b)&d))+e+k0+words[0];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|((~b)&d))+e+k0+words[1];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|((~b)&d))+e+k0+words[2];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|((~b)&d))+e+k0+words[3];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|((~b)&d))+e+k0+words[4];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|((~b)&d))+e+k0+words[5];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|((~b)&d))+e+k0+words[6];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|((~b)&d))+e+k0+words[7];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|((~b)&d))+e+k0+words[8];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|((~b)&d))+e+k0+words[9];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|((~b)&d))+e+k0+words[10];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|((~b)&d))+e+k0+words[11];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|((~b)&d))+e+k0+words[12];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|((~b)&d))+e+k0+words[13];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|((~b)&d))+e+k0+words[14];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|((~b)&d))+e+k0+words[15];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|((~b)&d))+e+k0+words[16];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|((~b)&d))+e+k0+words[17];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|((~b)&d))+e+k0+words[18];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|((~b)&d))+e+k0+words[19];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k1+words[20];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k1+words[21];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k1+words[22];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k1+words[23];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k1+words[24];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k1+words[25];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k1+words[26];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k1+words[27];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k1+words[28];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k1+words[29];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k1+words[30];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k1+words[31];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k1+words[32];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k1+words[33];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k1+words[34];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k1+words[35];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k1+words[36];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k1+words[37];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k1+words[38];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k1+words[39];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|(b&d)|(c&d))+e+k2+words[40];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|(b&d)|(c&d))+e+k2+words[41];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|(b&d)|(c&d))+e+k2+words[42];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|(b&d)|(c&d))+e+k2+words[43];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|(b&d)|(c&d))+e+k2+words[44];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|(b&d)|(c&d))+e+k2+words[45];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|(b&d)|(c&d))+e+k2+words[46];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|(b&d)|(c&d))+e+k2+words[47];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|(b&d)|(c&d))+e+k2+words[48];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|(b&d)|(c&d))+e+k2+words[49];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|(b&d)|(c&d))+e+k2+words[50];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|(b&d)|(c&d))+e+k2+words[51];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|(b&d)|(c&d))+e+k2+words[52];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|(b&d)|(c&d))+e+k2+words[53];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|(b&d)|(c&d))+e+k2+words[54];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|(b&d)|(c&d))+e+k2+words[55];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|(b&d)|(c&d))+e+k2+words[56];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|(b&d)|(c&d))+e+k2+words[57];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|(b&d)|(c&d))+e+k2+words[58];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+((b&c)|(b&d)|(c&d))+e+k2+words[59];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k3+words[60];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k3+words[61];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k3+words[62];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k3+words[63];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k3+words[64];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k3+words[65];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k3+words[66];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k3+words[67];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k3+words[68];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k3+words[69];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k3+words[70];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k3+words[71];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k3+words[72];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k3+words[73];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k3+words[74];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k3+words[75];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k3+words[76];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k3+words[77];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k3+words[78];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;
                temp = ROTL(a, 5)+(b^c^d)+e+k3+words[79];            
                e=d;d=c;c=ROTL(b,30);b=a;a=temp;

                uint offset = invocationID * 6;
                output_data[offset] = input_data[0] + a;
                output_data[offset + 1] = input_data[1] + b;
                output_data[offset + 2] = input_data[2] + c;
                output_data[offset + 3] = input_data[3] + d;
                output_data[offset + 4] = input_data[4] + e;
                output_data[offset + 5] = invocationID;
            }
        ",
    }
}