#include <iostream>
#include <cstdlib>
const size_t DEFAULT_CAPACITY = 4;

template<class T>
struct binary_heap {
	T* ptr;
    size_t len, cap;
    binary_heap() {
    	ptr=NULL;
    	this->clear();
	} 
	
	~binary_heap() {
		free(this->ptr);
	} 
    
    void push(T value) {
    	if (this->ptr==NULL||this->len==this->cap) 
    		this->buf_double();
		this->ptr[this->len]=value;
		size_t cur=this->len,fa;
		while(cur>0) {
			fa=(cur-1)/2;
			if(this->ptr[fa]>=this->ptr[cur]) break;
			this->swap(cur, fa);
			cur=fa;
		}
		this->len++;
    }
    
    void dump() {
		size_t j;
		printf("["); 
		for (j=0;j<this->len;++j) {
			printf("%ld, ", this->ptr[j]);
		}
		printf("]\n"); 
	}
    
    T pop() {
    	T ans=this->ptr[0];
    	this->len--;
    	this->swap(0, this->len);
    	size_t cur=0,nxt;
    	while(cur<this->len) {
    		nxt=cur*2+1;
    		if(nxt+1<this->len&&this->ptr[nxt]<this->ptr[nxt+1])
    			++nxt;
    		if(nxt>=this->len||this->ptr[cur]>=this->ptr[nxt])
    			break;
    		this->swap(cur, nxt);
    		cur=nxt;
		}
    	return ans;
	}
	
	T peek() {
		return this->ptr[0];
	}
	
	bool is_empty() {
		return this->len == 0;
	}
	
	void clear() {
		if(this->is_empty()) return;
    	this->len=0;
		this->cap=DEFAULT_CAPACITY;
		if(this->ptr!=NULL) free(this->ptr);
	}
    
    void buf_double() {
    	T* new_ptr;
		if(this->ptr!=NULL) {	
			(this->cap)*=2;
			new_ptr = (T*)malloc((this->cap)*sizeof(T));
			memcpy(new_ptr, this->ptr, (this->cap)*sizeof(T));
			free(this->ptr);
		} else {
			new_ptr = (T*)malloc((this->cap)*sizeof(T));
		}
		this->ptr = new_ptr;
	}
	
	void swap(size_t a, size_t b) {
		T c=*((this->ptr)+a);
		*((this->ptr)+a)=*((this->ptr)+b);
		*((this->ptr)+b)=c;
	}
};

int main() {
	binary_heap<int> heap;
	char c;int i;
	while(~scanf("%c",&c)) {
		if(c=='i') scanf("%d",&i), heap.push(i), getchar();
		if(c=='o') 
			if (heap.is_empty()) std::cout<<"pop: empty"<<std::endl;
			else std::cout<<heap.pop()<<std::endl;
		if(c=='c') heap.clear();
		if(c=='p') 
			if (heap.is_empty()) std::cout<<"peak: empty"<<std::endl;
			else std::cout<<heap.peek()<<std::endl;
		if(c=='d') 
			heap.dump();
			
	}
    return 0;
}
