
toc_ol=false;

function toc(){
  var tag,id,begin,end;
  var a = document.getElementsByTagName("h3");
  if(toc_ol){
    begin="<ol>";
    end="</ol>"; 
  }else{
    begin="<ul>";
    end="</ul>";
  }
  var b=[];
  b.push(begin);
  for(var i=0; i<a.length; i++){
    var h=a[i];
    var data = h.getAttribute("data-date");
    data = data.slice(0,4)+"-"+data.slice(4,6)+"-"+data.slice(6);
    var date="<span style='color: #90908a'>["+data+"]</span> ";
    if(h.id){
      b.push("<li>"+date+" <a href='#"+h.id+"'>"+h.innerHTML+"</a>");
    }else{
      b.push("<li>"+date+" "+h.innerHTML);
    }
    h.innerHTML=date+h.innerHTML;
  }
  b.push(end);
  var t = document.getElementById("toc");
  t.innerHTML=b.join("");
}

