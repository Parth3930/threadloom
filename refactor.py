import re

path = 'D:/framework/threadloom/template/src/pages/index/components/demo.rs'
with open(path, 'r', encoding='utf-8') as f:
    content = f.read()

pattern1 = r'div\(class="flex flex-col gap-6 bg-white dark:bg-gray-800 p-6 shadow-sm border border-gray-200 dark:border-gray-800 rounded-xl transition-colors duration-300 tl-card"\) \{\s*h3\(class="text-xl font-medium text-gray-800 dark:text-gray-100 border-b dark:border-gray-800 pb-2"\) \{ "(.*?)" \}'
content = re.sub(pattern1, r'Card(title="\1") {', content)

pattern2 = r'div\(class="flex flex-col gap-6 bg-white dark:bg-gray-800 p-6 shadow-sm border border-gray-200 dark:border-gray-800 rounded-xl transition-colors duration-300 tl-card md:col-span-2"\) \{\s*h3\(class="text-xl font-medium text-gray-800 dark:text-gray-100 border-b dark:border-gray-800 pb-2"\) \{ "(.*?)" \}'
content = re.sub(pattern2, r'Card(title="\1", wide=true) {', content)

card_code = '''
#[derive(Default)]
pub struct CardProps {
    pub title: String,
    pub wide: bool,
    pub children: Vec<View>,
}

#[allow(non_snake_case)]
pub fn Card(props: CardProps) -> View {
    let class = if props.wide {
        "flex flex-col gap-6 bg-white dark:bg-gray-800 p-6 shadow-sm border border-gray-200 dark:border-gray-800 rounded-xl transition-colors duration-300 tl-card md:col-span-2"
    } else {
        "flex flex-col gap-6 bg-white dark:bg-gray-800 p-6 shadow-sm border border-gray-200 dark:border-gray-800 rounded-xl transition-colors duration-300 tl-card"
    };
    let mut container = threadloom_core::element("div")
        .attr("class", class)
        .child(
            threadloom_core::element("h3")
                .attr("class", "text-xl font-medium text-gray-800 dark:text-gray-100 border-b dark:border-gray-800 pb-2")
                .child(threadloom_core::text(props.title))
        );
    for c in props.children {
        container = container.child(c);
    }
    container.into_view()
}
'''

content = content.replace('pub fn demo_component', card_code + '\npub fn demo_component')

with open(path, 'w', encoding='utf-8') as f:
    f.write(content)
